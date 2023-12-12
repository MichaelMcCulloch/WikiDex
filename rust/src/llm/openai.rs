use crate::llm::protocol::PartialLlmMessage;

use super::{AsyncLlmService, LlmMessage, LlmRole, LlmServiceError, ModelKind};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
    Chat, Client,
};
use backoff::future::retry;
use backoff::ExponentialBackoff;
use futures::StreamExt;
use url::Url;

use tokio::sync::mpsc::UnboundedSender;
pub(crate) struct AsyncOpenAiService {
    client: Client<OpenAIConfig>,
    model_name: String,
    model_kind: ModelKind,
}

#[async_trait::async_trait]
impl AsyncLlmService for AsyncOpenAiService {
    type E = LlmServiceError;
    async fn get_llm_answer(&self, system: String, query: String) -> Result<LlmMessage, Self::E> {
        match self.model_kind {
            ModelKind::Instruct => todo!(),
            ModelKind::Chat => {
                let request = self.create_chat_request(system, query, None)?;
                let response = self
                    .client
                    .chat()
                    .create(request)
                    .await
                    .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

                let response = response
                    .choices
                    .into_iter()
                    .next()
                    .ok_or(LlmServiceError::EmptyResponse)?;

                match (
                    LlmRole::from(&response.message.role),
                    response.message.content,
                ) {
                    (LlmRole::System, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::System)),
                    (LlmRole::Function, _) => {
                        Err(LlmServiceError::UnexpectedRole(LlmRole::Function))
                    }
                    (_, None) => Err(LlmServiceError::EmptyResponse),
                    (role, Some(content)) => Ok(LlmMessage { role, content }),
                }
            }
        }
    }
    async fn stream_llm_answer(
        &self,
        system: String,
        query: String,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E> {
        match self.model_kind {
            ModelKind::Instruct => todo!(),
            ModelKind::Chat => {
                let request = self.create_chat_request(system, query, None)?;

                let mut stream = self
                    .client
                    .chat()
                    .create_stream(request)
                    .await
                    .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

                let _ = stream.next().await; //Skip the first element
                while let Some(Ok(fragment)) = stream.next().await {
                    let response = fragment
                        .choices
                        .into_iter()
                        .next()
                        .ok_or(LlmServiceError::EmptyResponse)?;
                    if let Some(role) = response.delta.role {
                        tx.send(PartialLlmMessage {
                            role: Some(LlmRole::from(&role)),
                            content: None,
                        })
                        .unwrap();
                    }
                    if let Some(content) = response.delta.content {
                        let _ = tx.send(PartialLlmMessage {
                            role: None,
                            content: Some(content),
                        });
                    }
                }

                Ok(())
            }
        }
    }
    async fn wait_for_service(&self) -> Result<(), LlmServiceError> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1u16)
            .model(self.model_name.clone())
            .messages(vec![])
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        retry(ExponentialBackoff::default(), || async {
            Ok(self.client.chat().create(request.clone()).await?)
        })
        .await
        .map_err(LlmServiceError::AsyncOpenAiError)?;

        Ok(())
    }
}

impl AsyncOpenAiService {
    pub(crate) fn new<S: AsRef<str>>(
        openai_key: Option<String>,
        host: Url,
        model_name: S,
        model_kind: ModelKind,
    ) -> Self {
        let openai_config = match openai_key {
            Some(key) => OpenAIConfig::new().with_api_key(key),
            None => OpenAIConfig::new().with_api_base(host),
        };

        let client = Client::with_config(openai_config);
        let model_name = model_name.as_ref().to_string();
        Self {
            client,
            model_name,
            model_kind,
        }
    }
    fn create_chat_request(
        &self,
        system: String,
        query: String,
        max_new_tokens: Option<u16>,
    ) -> Result<CreateChatCompletionRequest, <Self as AsyncLlmService>::E> {
        let query = format!("Obey the instructions in the system prompt. You must cite every statement [1] and provide your answer in a long-form essay, formatted as markdown. Delimite the essay from the reference list with exactly the line '================'\n{query}");

        let system = ChatCompletionRequestSystemMessageArgs::default()
            .content(system)
            .build()
            .map(|e| ChatCompletionRequestMessage::System(e))
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let query = ChatCompletionRequestUserMessageArgs::default()
            .content(query)
            .build()
            .map(|e| ChatCompletionRequestMessage::User(e))
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let message_openai_compat = vec![system, query];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_new_tokens.unwrap_or(2048u16))
            .model(self.model_name.clone())
            .messages(message_openai_compat)
            .stop("================")
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        Ok(request)
    }

    // fn create_chat_request(
    //     &self,
    //     system: String,
    //     query: String,
    //     max_new_tokens: Option<u16>,
    // ) -> Result<CreateCompletionRequest, <Self as AsyncLlmService>::E> {
    //     let input = LlmChatInput {
    //         system,
    //         conversation: vec![LlmMessage {
    //             role: LlmRole::User,
    //             content: format!("Obey the instructions in the system prompt. You must cite every statement [1] and provide your answer in a long-form essay, formatted as markdown. Delimite the essay from the reference list with exactly the line '================'\n{query}"),
    //         }],
    //     };
    //     let message_openai_compat: Result<
    //         Vec<ChatCompletionRequestMessage>,
    //         <AsyncOpenAiService as AsyncLlmService>::E,
    //     > = input.into();
    //     let request = CreateChatCompletionRequestArgs::default()
    //         .max_tokens(max_new_tokens.unwrap_or(2048u16))
    //         .model(self.model_name.clone())
    //         .messages(message_openai_compat?)
    //         .stop("================")
    //         .build()
    //         .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

    //     Ok(request)
    // }
}
