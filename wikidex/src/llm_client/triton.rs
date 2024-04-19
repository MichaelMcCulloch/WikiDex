use anyhow::Context;
use tokio::sync::mpsc::UnboundedSender;

use super::{
    error::LlmClientError,
    triton_helper::{create_request, deserialize_bytes_tensor},
    LanguageServiceArguments, LlmClient, LlmClientBackend, LlmClientBackendKind, LlmClientService,
    TritonClient,
};
use async_stream::stream;

use trtllm::triton::grpc_inference_service_client::GrpcInferenceServiceClient;

impl LlmClient<TritonClient> {
    pub(crate) async fn new<S: AsRef<str>>(triton_url: S) -> Result<Self, LlmClientError> {
        let client = GrpcInferenceServiceClient::connect(String::from(triton_url.as_ref())).await?;
        Ok(Self { client })
    }
}
impl LlmClientBackendKind for TritonClient {}
impl LlmClientBackend for LlmClient<TritonClient> {
    async fn get_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError> {
        let prompt = self.fill_rag_template(arguments);
        let request = create_request(prompt, false, max_tokens, stop_phrases)?;
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

    async fn stream_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        let prompt = self.fill_rag_template(arguments);
        log::info!("{prompt}");
        let request = create_request(prompt, true, max_tokens, stop_phrases)?;
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
