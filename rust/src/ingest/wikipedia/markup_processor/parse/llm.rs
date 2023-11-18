use std::cmp::min;

use crate::{
    ingest::wikipedia::helper::wiki::{DescribedTable, UnlabledDocument},
    llm::{LlmInput, LlmMessage, LlmRole, LlmService, LlmServiceError, OpenAiService},
};

const ESTIMATED_CONTROL_TOKENS_IN_PROMPT: usize = 10;
const ROOM_FOR_SUMMARY: usize = 2048;

pub(crate) async fn process_table_to_llm(
    table: &str,
    client: &OpenAiService,
) -> Result<UnlabledDocument, LlmServiceError> {
    let system = String::from("Interpret and summarize the following (possibly truncated) HTML table in a concise, plain English description.");

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
            message: table_str_chars[0..lim].into_iter().collect::<String>(),
            role: LlmRole::User,
        }],
    };

    let output = client.get_llm_answer(message, Some(2048)).await;

    let output = output?;
    let response = output
        .conversation
        .into_iter()
        .last()
        .and_then(|m| Some(m.message))
        .ok_or(LlmServiceError::EmptyResponse)?;

    Ok(UnlabledDocument::from_str_and_vec(
        String::new(),
        vec![DescribedTable {
            description: response,
            table: table.to_string(),
        }],
    ))
}
