use std::{cmp::min, time::Instant};

use crate::llm::{
    LlmInput, LlmMessage, LlmRole, LlmServiceError, SyncLlmService, SyncOpenAiService,
};

const ESTIMATED_CONTROL_TOKENS_IN_PROMPT: usize = 30;
const ROOM_FOR_SUMMARY: usize = 8192;

pub(crate) fn process_table_to_llm(table_for_summary: &str) -> Result<String, LlmServiceError> {
    let system_description = format!("You are a helpful assistant that describes the purpose of the table based on the headers and a random subset of rows from the table.");

    let message_description = LlmInput {
        system: system_description,
        conversation: vec![LlmMessage {
            content: table_for_summary.to_string(),
            role: LlmRole::User,
        }],
    };
    let description = client
        .get_llm_answer(message_description, Some((ROOM_FOR_SUMMARY) as u16))
        .and_then(|m| {
            if m.content.is_empty() || m.content == "\n" {
                log::error!("{}", LlmServiceError::EmptyResponse);
                Err(LlmServiceError::EmptyResponse)
            } else {
                Ok(m.content)
            }
        })
        .map_err(|e| {
            log::error!("{e}");
            e
        })?;

    Ok(description)
}
