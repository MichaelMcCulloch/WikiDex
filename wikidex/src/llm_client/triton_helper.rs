use std::str;
use std::str::Utf8Error;

use anyhow::Context;
use bytes::{Buf, Bytes};
use trtllm::triton::request::{Builder, InferTensorData};

pub fn deserialize_bytes_tensor(encoded_tensor: Vec<u8>) -> Result<Vec<String>, Utf8Error> {
    let mut bytes = Bytes::from(encoded_tensor);
    let mut strs = Vec::new();
    while bytes.has_remaining() {
        let len = bytes.get_u32_le() as usize;
        if len <= bytes.remaining() {
            let slice = bytes.split_to(len);
            let s = str::from_utf8(&slice)?;
            strs.push(s.to_string());
        }
    }
    Ok(strs)
}

pub(crate) fn create_request<S: AsRef<str>>(
    prompt: String,
    stream: bool,
    max_tokens: u16,
    stop_phrases: Vec<S>,
) -> Result<trtllm::triton::ModelInferRequest, anyhow::Error> {
    Builder::new()
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
        .input("stream", [1, 1], InferTensorData::Bool(vec![stream]))
        .output("text_output")
        .build()
        .context("Failed")
}