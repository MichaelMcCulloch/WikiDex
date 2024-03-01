use super::{delegate::LanguageServiceArguments, error::LlmServiceError};
use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{CreateCompletionRequest, CreateCompletionRequestArgs, ListModelResponse},
    Client,
};
use futures::StreamExt;

use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct InstructClient {
    instruct_client: Client<OpenAIConfig>,
    instruct_model_name: String,
}

impl InstructClient {
    pub(crate) async fn up(&self) -> Result<ListModelResponse, OpenAIError> {
        self.instruct_client.models().list().await
    }

    pub(super) fn new(instruct_client: Client<OpenAIConfig>, instruct_model_name: String) -> Self {
        InstructClient {
            instruct_client,
            instruct_model_name,
        }
    }

    pub(crate) async fn get_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<String, LlmServiceError> {
        let request = self.create_instruct_request(arguments, max_tokens, stop_phrases)?;

        let response = self
            .instruct_client
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

    pub(crate) async fn stream_response(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
        max_tokens: u16,
        stop_phrases: Vec<&str>,
    ) -> Result<(), LlmServiceError> {
        let request = self.create_instruct_request(arguments, max_tokens, stop_phrases)?;

        let mut stream = self
            .instruct_client
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

pub(crate) trait InstructRequest {
    fn create_instruct_request<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<CreateCompletionRequest, LlmServiceError>;
}

impl InstructRequest for InstructClient {
    fn create_instruct_request<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments,
        max_tokens: u16,
        stop_phrases: Vec<S>,
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
            .max_tokens(max_tokens)
            .model(&self.instruct_model_name)
            .n(1)
            .prompt(query)
            .stop(stop_phrases.iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .build()
            .map_err(LlmServiceError::AsyncOpenAiError)?;
        Ok(request)
    }
}
