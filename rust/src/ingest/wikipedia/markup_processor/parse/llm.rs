use crate::{
    ingest::wikipedia::helper::wiki::UnlabledDocument,
    llm::{LlmInput, LlmService, LlmServiceError, OpenAiService},
};

pub(crate) async fn process_table_to_llm(
    table: &str,
    client: &OpenAiService,
) -> Result<UnlabledDocument, LlmServiceError> {
    let message = LlmInput {
        system: todo!(),
        conversation: todo!(),
    };
    client.get_llm_answer(message).await?;
}
