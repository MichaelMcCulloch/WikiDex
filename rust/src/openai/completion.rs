use super::{
    delegate::LanguageServiceServiceArguments, error::LlmServiceError, service::TCompletionClient,
};
use async_openai::{
    config::OpenAIConfig,
    types::{CreateCompletionRequest, CreateCompletionRequestArgs},
    Client,
};
use futures::StreamExt;

use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct CompletionClient {
    completion_client: Client<OpenAIConfig>,
    completion_model_name: String,
}

impl CompletionClient {
    pub(super) fn new(
        completion_client: Client<OpenAIConfig>,
        completion_model_name: String,
    ) -> Self {
        CompletionClient {
            completion_client,
            completion_model_name,
        }
    }
}

impl TCompletion for CompletionClient {
    fn create_instruct_request(
        &self,
        arguments: LanguageServiceServiceArguments,
    ) -> Result<CreateCompletionRequest, LlmServiceError> {
        let c1 = arguments.citation_index_begin + 1;
        let c2 = arguments.citation_index_begin + 2;
        let c3 = arguments.citation_index_begin + 3;
        let c4 = arguments.citation_index_begin + 4;

        let query = arguments
            .system
            .replace("___USER_QUERY___", arguments.query)
            .replace("___URL___", "http://localhost")
            .replace("___CITE1___", &c1.to_string())
            .replace("___CITE2___", &c2.to_string())
            .replace("___CITE3___", &c3.to_string())
            .replace("___CITE4___", &c4.to_string())
            .replace("___DOCUMENT_LIST___", arguments.documents);

        let request = CreateCompletionRequestArgs::default()
            .max_tokens(2048u16)
            .model(&self.completion_model_name)
            .n(1)
            .prompt(query)
            .stop("References:")
            .build()
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        Ok(request)
    }
}

#[async_trait::async_trait]
impl TCompletionClient for CompletionClient {
    async fn get_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<String, LlmServiceError> {
        let request = self.create_instruct_request(arguments)?;
        let response = self
            .completion_client
            .completions()
            .create(request)
            .await
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        let response = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmServiceError::EmptyResponse)?;
        Ok(response.text)
    }

    async fn stream_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError> {
        let request = self.create_instruct_request(arguments)?;

        let mut stream = self
            .completion_client
            .completions()
            .create_stream(request)
            .await
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        while let Some(Ok(fragment)) = stream.next().await {
            let response = fragment
                .choices
                .into_iter()
                .next()
                .ok_or(LlmServiceError::EmptyResponse)?;

            let _ = tx.send(response.text);
        }

        Ok(())
    }
}

pub(crate) trait TCompletion {
    fn create_instruct_request(
        &self,
        arguments: LanguageServiceServiceArguments,
    ) -> Result<CreateCompletionRequest, LlmServiceError>;
}
