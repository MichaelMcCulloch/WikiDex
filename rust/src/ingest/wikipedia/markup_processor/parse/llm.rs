use std::{cmp::min, time::Instant};

use crate::llm::{
    LlmInput, LlmMessage, LlmRole, LlmServiceError, SyncLlmService, SyncOpenAiService,
};

const ESTIMATED_CONTROL_TOKENS_IN_PROMPT: usize = 30;
const ROOM_FOR_SUMMARY: usize = 512;

pub(crate) fn process_table_to_llm(
    table: &str,
    client: &SyncOpenAiService,
) -> Result<String, LlmServiceError> {
    // let system = format!("You are a helpful assistant that transforms the data provided into a concise summary paragraph conveying only the purpose of the data, based on the header and the first three rows. This table is incomplete.");
    let system = format!("You are a helpful assistant that describes the purpose of the table based on the headers and a random subset of rows from the table.");

    let table_str_chars = table.chars().collect::<Vec<_>>();

    let lim = min(
        table_str_chars.len(),
        client.model_context_length
            - ROOM_FOR_SUMMARY
            - system.len()
            - ESTIMATED_CONTROL_TOKENS_IN_PROMPT,
    );

    let message = LlmInput {
        system,
        conversation: vec![LlmMessage {
            content: table_str_chars[0..lim].into_iter().collect::<String>(),
            role: LlmRole::User,
        }],
    };

    let start = Instant::now();

    let description = client
        .get_llm_answer(message, Some(ROOM_FOR_SUMMARY as u16))
        .and_then(|m| {
            if m.content.is_empty() || m.content == "\n" {
                log::error!("{}", LlmServiceError::EmptyResponse);
                Err(LlmServiceError::EmptyResponse)
            } else {
                log::info!("{:?} :: {} :: {}", start.elapsed(), table, m.content);
                Ok(m.content)
            }
        })
        .map_err(|e| {
            log::error!("{e}");
            e
        })?;

    Ok(description)
}
