use anyhow::Context;
use tokio::sync::mpsc::UnboundedSender;

use super::{
    error::LlmClientError, LanguageServiceArguments, LlmClient, LlmClientBackend,
    LlmClientBackendKind, LlmClientService, TritonClient,
};
use async_stream::stream;

use trtllm::{
    triton::{
        grpc_inference_service_client::GrpcInferenceServiceClient,
        request::{Builder, InferTensorData},
    },
    utils::deserialize_bytes_tensor,
};

impl LlmClient<TritonClient> {
    pub(crate) async fn new<S: AsRef<str>>(triton_url: S) -> Result<Self, LlmClientError> {
        let client = GrpcInferenceServiceClient::connect(String::from(triton_url.as_ref()))
            .await
            .map_err(LlmClientError::TonicError)?;
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
        let request = Builder::new()
            .model_name("ensemble".to_string())
            .input(
                "text_input",
                [1, 1],
                InferTensorData::Bytes(vec![prompt.as_bytes().to_vec()]),
            )
            .input(
                "max_tokens",
                [1, 1],
                InferTensorData::Int32(vec![max_tokens as i32]),
            )
            .input(
                "bad_words",
                [1, 1],
                InferTensorData::Bytes(vec!["".as_bytes().to_vec()]),
            )
            .input(
                "stop_words",
                [1, 1],
                InferTensorData::Bytes(
                    stop_phrases
                        .into_iter()
                        .map(|s| s.as_ref().to_string().into_bytes())
                        .collect(),
                ),
            )
            .input("top_p", [1, 1], InferTensorData::FP32(vec![1.0f32]))
            .input("temperature", [1, 1], InferTensorData::FP32(vec![1.0f32]))
            .input(
                "presence_penalty",
                [1, 1],
                InferTensorData::FP32(vec![0.0f32]),
            )
            .input("beam_width", [1, 1], InferTensorData::Int32(vec![1i32]))
            .input("stream", [1, 1], InferTensorData::Bool(vec![true]))
            .output("text_output")
            .build()
            .context("Failed")
            .map_err(LlmClientError::Anyhow)?;
        let request = stream! { yield request };
        let request = tonic::Request::new(request);

        let mut stream = self
            .client
            .clone()
            .model_stream_infer(request)
            .await
            .context("failed to call triton grpc method model_stream_infer")
            .map_err(LlmClientError::Anyhow)?
            .into_inner();

        let mut contents: String = String::new();
        while let Some(response) = stream
            .message()
            .await
            .map_err(LlmClientError::TonicStatus)?
        {
            if !response.error_message.is_empty() {
                return Err(LlmClientError::EmptyResponse);
            }
            let infer_response = response
                .infer_response
                .context("empty infer response received")
                .map_err(LlmClientError::Anyhow)?;

            let raw_content = infer_response.raw_output_contents[0].clone();
            let content = deserialize_bytes_tensor(raw_content)
                .map_err(LlmClientError::Utf8Error)?
                .into_iter()
                .collect();

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

        let request = Builder::new()
            .model_name("tensorrt_llm_bls".to_string())
            .input(
                "text_input",
                [1, 1],
                InferTensorData::Bytes(vec![prompt.as_bytes().to_vec()]),
            )
            .input(
                "max_tokens",
                [1, 1],
                InferTensorData::Int32(vec![max_tokens as i32]),
            )
            .input(
                "bad_words",
                [1, 1],
                InferTensorData::Bytes(vec!["".as_bytes().to_vec()]),
            )
            .input(
                "stop_words",
                [1, 1],
                InferTensorData::Bytes(
                    stop_phrases
                        .into_iter()
                        .map(|s| s.as_ref().to_string().into_bytes())
                        .collect(),
                ),
            )
            .input("top_p", [1, 1], InferTensorData::FP32(vec![1.0f32]))
            .input("temperature", [1, 1], InferTensorData::FP32(vec![1.0f32]))
            .input(
                "presence_penalty",
                [1, 1],
                InferTensorData::FP32(vec![0.0f32]),
            )
            .input("beam_width", [1, 1], InferTensorData::Int32(vec![1i32]))
            .input("stream", [1, 1], InferTensorData::Bool(vec![true]))
            .output("text_output")
            .build()
            .context("Failed")
            .map_err(LlmClientError::Anyhow)?;
        let request = stream! { yield request };
        let request = tonic::Request::new(request);
        let mut stream = self
            .client
            .clone()
            .model_stream_infer(request)
            .await
            .context("failed to call triton grpc method model_stream_infer")
            .map_err(LlmClientError::Anyhow)?
            .into_inner();
        let mut previous_response = String::new();
        while let Some(response) = stream
            .message()
            .await
            .map_err(LlmClientError::TonicStatus)?
        {
            if !response.error_message.is_empty() {
                // Corresponds to https://github.com/openai/openai-python/blob/17ac6779958b2b74999c634c4ea4c7b74906027a/src/openai/_streaming.py#L113

                return Err(LlmClientError::EmptyResponse);
            }
            let infer_response = response
                .infer_response
                .context("empty infer response received")
                .map_err(LlmClientError::Anyhow)?;

            let raw_content = infer_response.raw_output_contents[0].clone();
            let content = deserialize_bytes_tensor(raw_content)
                .map_err(LlmClientError::Utf8Error)?
                .into_iter()
                .collect::<String>();
            let new_response = content.clone();
            // https://github.com/triton-inference-server/tensorrtllm_backend/issues/112
            let content = match content.as_str().strip_prefix(&previous_response) {
                Some(content) => content,
                None => content.as_str(),
            };
            previous_response = new_response;

            if !content.is_empty() {
                let _ = tx.send(content.to_string());
            }
        }
        Ok(())
    }
}
