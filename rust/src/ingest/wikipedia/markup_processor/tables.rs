use std::collections::HashSet;

use async_openai::{config::OpenAIConfig, Client};
use parse_wiki_text::*;

use crate::{
    ingest::wikipedia::helper::wiki::{DescribedTable, UnlabledDocument},
    llm::OpenAiService,
};

use super::{
    nodes::nodes_to_string, regexes::Regexes, Process, WikiMarkupProcessingError,
    WikiMarkupProcessor,
};
pub(super) async fn table_captions_to_string(
    table_captions: &Vec<TableCaption<'_>>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for tc in table_captions.iter() {
        documents.push(table_caption_to_string(tc, regexes, client).await?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) async fn table_rows_to_string(
    table_rows: &Vec<TableRow<'_>>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for tr in table_rows.iter() {
        documents.push(table_row_to_string(tr, regexes, client).await?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) async fn table_cells_to_string(
    table_cells: &Vec<TableCell<'_>>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents: Vec<UnlabledDocument> = vec![];
    for tc in table_cells.iter() {
        documents.push(table_cell_to_string(tc, regexes, client).await?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) async fn table_cell_to_string(
    TableCell { content, type_, .. }: &TableCell<'_>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let content = nodes_to_string(content, regexes, client).await?;
    let content = content.trim();
    if content.document.is_empty() {
        Ok(UnlabledDocument::new())
    } else {
        let tag = match type_ {
            TableCellType::Heading => "th",
            TableCellType::Ordinary => "td",
        };
        Ok(UnlabledDocument::from_str_and_vec(
            format!("\t\t<{tag}>{}</{tag}>\n", content.document),
            content.table,
        ))
    }
}
pub(super) async fn table_row_to_string(
    TableRow { cells, .. }: &TableRow<'_>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let cells = table_cells_to_string(cells, regexes, client).await?;
    if cells.document.is_empty() {
        Ok(UnlabledDocument::new())
    } else {
        Ok(UnlabledDocument::from_str_and_vec(
            format!("\t<tr>\n{}\t</tr>\n", cells.document),
            cells.table,
        ))
    }
}
pub(super) async fn table_caption_to_string(
    TableCaption { content, .. }: &TableCaption<'_>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    nodes_to_string(content, regexes, client).await
}
