use std::cmp::min;

use crate::{
    ingest::wikipedia::helper::wiki::{DescribedTable, UnlabledDocument},
    llm::{LlmInput, LlmMessage, LlmRole, LlmServiceError, SyncLlmService, SyncOpenAiService},
};

const ESTIMATED_CONTROL_TOKENS_IN_PROMPT: usize = 10;
const ROOM_FOR_SUMMARY: usize = 2048;

pub(crate) fn process_table_to_llm(
    table: &str,
    client: &SyncOpenAiService,
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
            content: table_str_chars[0..lim].into_iter().collect::<String>(),
            role: LlmRole::User,
        }],
    };

    let output = client.get_llm_answer(message, Some(2048));

    let description = output.and_then(|m| Ok(m.content))?;

    Ok(UnlabledDocument::from_str_and_vec(
        String::new(),
        vec![DescribedTable {
            description,
            table: table.to_string(),
        }],
    ))
}
