use std::cmp::min;

use parse_wiki_text::{TableCaption, TableCell, TableCellType, TableRow};

use crate::{
    ingest::wikipedia::{
        helper::wiki::{DescribedTable, UnlabledDocument},
        markup_processor::{Process, WikiMarkupProcessingError::LlmError},
        WikiMarkupProcessor,
    },
    llm::SyncOpenAiService,
};

use super::{
    llm::process_table_to_llm,
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn table_to_string(
    captions: &Vec<parse_wiki_text::TableCaption<'_>>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
    rows: &Vec<parse_wiki_text::TableRow<'_>>,
) -> Result<UnlabledDocument, crate::ingest::wikipedia::markup_processor::WikiMarkupProcessingError>
{
    let captions = table_captions_to_string(captions, regexes, client)?;

    if let Some((rows, rows_for_summary)) = table_rows_to_string(rows, regexes, client)? {
        let table = if captions.document.is_empty() {
            format!("\n<table>\n{}</table>\n", rows.document)
        } else {
            format!(
                "\n<table caption='{}'>\n{}</table>\n",
                captions.document, rows.document
            )
        };
        let table_for_summary = if captions.document.is_empty() {
            format!("\n<table>\n{}</table>\n", rows_for_summary.document)
        } else {
            format!(
                "\n<table caption='{}'>\n{}</table>\n",
                captions.document, rows_for_summary.document
            )
        };
        let description = process_table_to_llm(&table_for_summary, client).map_err(LlmError)?;
        Ok(UnlabledDocument::from_str_and_vec(
            String::new(),
            vec![DescribedTable {
                description,
                table: table.to_string(),
            }],
        ))
    } else {
        Ok(UnlabledDocument::new())
    }
}

pub(super) fn table_captions_to_string(
    table_captions: &Vec<TableCaption<'_>>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let mut documents = vec![];
    for tc in table_captions.iter() {
        documents.push(table_caption_to_string(tc, regexes, client)?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) fn table_rows_to_string(
    table_rows: &Vec<TableRow<'_>>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<Option<(UnlabledDocument, UnlabledDocument)>, <WikiMarkupProcessor as Process>::E> {
    if table_rows.is_empty() {
        Ok(None)
    } else {
        let mut documents = vec![];
        for tr in table_rows.iter() {
            documents.push(table_row_to_string(tr, regexes, client)?)
        }

        let lim: usize = min(documents.len(), 50);
        let docs_for_summary = documents[0..lim].to_vec();
        Ok(Some((
            UnlabledDocument::join_all(documents, &""),
            UnlabledDocument::join_all(docs_for_summary, &""),
        )))
    }
}

pub(super) fn table_cells_to_string(
    table_cells: &Vec<TableCell<'_>>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let mut documents: Vec<UnlabledDocument> = vec![];
    for tc in table_cells.iter() {
        documents.push(table_cell_to_string(tc, regexes, client)?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) fn table_cell_to_string(
    TableCell { content, type_, .. }: &TableCell<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let content = nodes_to_string(content, regexes, client)?;
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
pub(super) fn table_row_to_string(
    TableRow { cells, .. }: &TableRow<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let cells = table_cells_to_string(cells, regexes, client)?;
    if cells.document.is_empty() {
        Ok(UnlabledDocument::new())
    } else {
        Ok(UnlabledDocument::from_str_and_vec(
            format!("\t<tr>\n{}\t</tr>\n", cells.document),
            cells.table,
        ))
    }
}
pub(super) fn table_caption_to_string(
    TableCaption { content, .. }: &TableCaption<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    nodes_to_string(content, regexes, client)
}
