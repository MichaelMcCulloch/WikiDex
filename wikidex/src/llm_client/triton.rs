use std::sync::Arc;

use anyhow::Context;
use tera::Tera;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use super::{
    error::LlmClientError,
    triton_helper::{create_request, deserialize_bytes_tensor},
    LanguageServiceArguments, LlmClient, LlmClientBackend, LlmClientBackendKind, TritonClient,
};
use async_stream::stream;

impl LlmClient<TritonClient> {
    pub(crate) fn new(client: TritonClient, tera: Arc<RwLock<Tera>>) -> Self {
        Self { client, tera }
    }
}
impl LlmClientBackendKind for TritonClient {}
impl LlmClientBackend for LlmClient<TritonClient> {
    async fn get_response(
        &self,
        arguments: LanguageServiceArguments,
    ) -> Result<String, LlmClientError> {
        let prompt = self
            .format_rag_template(
                &arguments.messages,
                &arguments.documents,
                &arguments.user_query,
            )
            .await?;
        let request = create_request(prompt, false, arguments.max_tokens, arguments.stop_phrases)?;
        let request = stream! { yield request };
        let request = tonic::Request::new(request);

        let mut stream = self
            .client
            .clone()
            .model_stream_infer(request)
            .await
            .context("failed to call triton grpc method model_stream_infer")?
            .into_inner();

        let mut contents: String = String::new();
        while let Some(response) = stream.message().await? {
            if !response.error_message.is_empty() {
                break;
            }
            let infer_response = response
                .infer_response
                .context("empty infer response received")?;

            let raw_content = infer_response.raw_output_contents[0].clone();
            let content = deserialize_bytes_tensor(raw_content)?.into_iter().collect();

            contents = content;
        }

        Ok(contents)
    }

    async fn stream_response(
        &self,
        arguments: LanguageServiceArguments,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmClientError> {
        let prompt = self
            .format_rag_template(
                &arguments.messages,
                &arguments.documents,
                &arguments.user_query,
            )
            .await?;
        let request = create_request(prompt, true, arguments.max_tokens, arguments.stop_phrases)?;
        let request = stream! { yield request };
        let request = tonic::Request::new(request);
        let mut stream = self
            .client
            .clone()
            .model_stream_infer(request)
            .await
            .context("failed to call triton grpc method model_stream_infer")?
            .into_inner();
        while let Some(response) = stream.message().await? {
            if !response.error_message.is_empty() {
                break;
            }
            let infer_response = response
                .infer_response
                .context("empty infer response received")?;

            let raw_content = infer_response.raw_output_contents[0].clone();
            let content = deserialize_bytes_tensor(raw_content)?
                .into_iter()
                .collect::<String>();

            if !content.is_empty() {
                let _ = tx.send(content.to_string());
            }
        }
        Ok(())
    }
}
