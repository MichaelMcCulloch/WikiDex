use std::cmp::min;

use parse_wiki_text::{TableCaption, TableCell, TableCellType, TableRow};

use crate::{
    ingest::wikipedia::{
        markup_processor::{
            Process,
            WikiMarkupProcessingError::{self, LlmError},
        },
        WikiMarkupProcessor,
    },
    llm::SyncOpenAiService,
};

use super::{
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn table_to_string(
    captions: &Vec<parse_wiki_text::TableCaption<'_>>,
    regexes: &Regexes,

    rows: &Vec<parse_wiki_text::TableRow<'_>>,
) -> Result<String, WikiMarkupProcessingError> {
    let captions = table_captions_to_string(captions, regexes)?;

    if let Some(rows) = table_rows_to_string(rows, regexes)? {
        let table = if captions.is_empty() {
            format!("\n<table>\n{}</table>\n", rows)
        } else {
            format!("\n<table caption='{}'>\n{}</table>\n", captions, rows)
        };

        Ok(table.to_string())
    } else {
        Ok(String::new())
    }
}

pub(super) fn table_captions_to_string(
    table_captions: &Vec<TableCaption<'_>>,
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];
    for tc in table_captions.iter() {
        documents.push(table_caption_to_string(tc, regexes)?)
    }
    Ok(documents.join(""))
}

pub(super) fn table_rows_to_string(
    table_rows: &Vec<TableRow<'_>>,
    regexes: &Regexes,
) -> Result<Option<String>, <WikiMarkupProcessor as Process>::E> {
    if table_rows.is_empty() {
        Ok(None)
    } else {
        let mut documents = vec![];
        for tr in table_rows.iter() {
            documents.push(table_row_to_string(tr, regexes)?)
        }

        Ok(Some(documents.join("")))
    }
}

pub(super) fn table_cells_to_string(
    table_cells: &Vec<TableCell<'_>>,
    regexes: &Regexes,
) -> ParseResult {
    let mut documents: Vec<String> = vec![];
    for tc in table_cells.iter() {
        documents.push(table_cell_to_string(tc, regexes)?)
    }
    Ok(documents.join(""))
}

pub(super) fn table_cell_to_string(
    TableCell { content, type_, .. }: &TableCell<'_>,
    regexes: &Regexes,
) -> ParseResult {
    let content = nodes_to_string(content, regexes)?;
    let content = content.trim();
    if content.is_empty() {
        Ok(String::new())
    } else {
        let tag = match type_ {
            TableCellType::Heading => "th",
            TableCellType::Ordinary => "td",
        };
        Ok(format!("\t\t<{tag}>{}</{tag}>\n", content))
    }
}
pub(super) fn table_row_to_string(
    TableRow { cells, .. }: &TableRow<'_>,
    regexes: &Regexes,
) -> ParseResult {
    let cells = table_cells_to_string(cells, regexes)?;
    if cells.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("\t<tr>\n{}\t</tr>\n", cells))
    }
}
pub(super) fn table_caption_to_string(
    TableCaption { content, .. }: &TableCaption<'_>,
    regexes: &Regexes,
) -> ParseResult {
    nodes_to_string(content, regexes)
}
