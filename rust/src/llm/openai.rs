use crate::llm::protocol::PartialLlmMessage;

use super::{
    AsyncLlmService, AsyncLlmServiceArguments, LlmMessage, LlmRole, LlmServiceError, ModelKind,
};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, CreateCompletionRequest, CreateCompletionRequestArgs,
    },
    Client,
};
use backoff::future::retry;
use backoff::ExponentialBackoff;
use futures::StreamExt;
use url::Url;

use tokio::sync::mpsc::UnboundedSender;

const PROMPT_SALT: &str = "";

pub(crate) struct AsyncOpenAiService {
    client: Client<OpenAIConfig>,
    model_name: String,
    model_kind: ModelKind,
}

#[async_trait::async_trait]
impl AsyncLlmService for AsyncOpenAiService {
    type E = LlmServiceError;
    async fn get_llm_answer(
        &self,
        arguments: AsyncLlmServiceArguments<'async_trait>,
    ) -> Result<LlmMessage, Self::E> {
        match self.model_kind {
            ModelKind::Instruct => {
                let request = self.create_instruct_request(arguments, None)?;
                let response = self
                    .client
                    .completions()
                    .create(request)
                    .await
                    .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

                let response = response
                    .choices
                    .into_iter()
                    .next()
                    .ok_or(LlmServiceError::EmptyResponse)?;

                Ok(LlmMessage {
                    role: LlmRole::Assistant,
                    content: response.text,
                })
            }
            ModelKind::Chat => {
                let request = self.create_chat_request(arguments, None)?;
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
        arguments: AsyncLlmServiceArguments<'async_trait>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E> {
        match self.model_kind {
            ModelKind::Instruct => {
                let request = self.create_instruct_request(arguments, None)?;

                let mut stream = self
                    .client
                    .completions()
                    .create_stream(request)
                    .await
                    .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

                while let Some(Ok(fragment)) = stream.next().await {
                    let response = fragment
                        .choices
                        .into_iter()
                        .next()
                        .ok_or(LlmServiceError::EmptyResponse)?;

                    let _ = tx.send(PartialLlmMessage {
                        role: None,
                        content: Some(response.text),
                    });
                }

                Ok(())
            }
            ModelKind::Chat => {
                let request = self.create_chat_request(arguments, None)?;

                let mut stream = self
                    .client
                    .chat()
                    .create_stream(request)
                    .await
                    .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

                let _ = stream.next().await;
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
        arguments: AsyncLlmServiceArguments,
        max_new_tokens: Option<u16>,
    ) -> Result<CreateChatCompletionRequest, <Self as AsyncLlmService>::E> {
        let query = format!("{PROMPT_SALT}\n{}", arguments.query);

        let system = arguments
            .system
            .replace("___DOCUMENT_LIST___", &arguments.documents);

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
            .stop("References")
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        Ok(request)
    }

    fn create_instruct_request(
        &self,
        arguments: AsyncLlmServiceArguments,
        max_new_tokens: Option<u16>,
    ) -> Result<CreateCompletionRequest, <Self as AsyncLlmService>::E> {
        let c1 = arguments.citation_index_begin + 1;
        let c2 = arguments.citation_index_begin + 2;
        let c3 = arguments.citation_index_begin + 3;
        let c4 = arguments.citation_index_begin + 4;

        let query = arguments
            .system
            .replace("___USER_QUERY___", &arguments.query)
            .replace("___URL___", &"http://localhost")
            .replace("___CITE1___", &c1.to_string())
            .replace("___CITE2___", &c2.to_string())
            .replace("___CITE3___", &c3.to_string())
            .replace("___CITE4___", &c4.to_string())
            .replace("___DOCUMENT_LIST___", &arguments.documents);

        log::info!("{query}");

        let request = CreateCompletionRequestArgs::default()
            .max_tokens(max_new_tokens.unwrap_or(2048u16))
            .model(self.model_name.clone())
            .n(1)
            .prompt(query)
            .stop("References")
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        Ok(request)
    }
}
