use parse_wiki_text::{TableCaption, TableCell, TableCellType, TableRow};

use crate::ingest::pipeline::wikipedia::WikiMarkupProcessingError;

use super::{
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn table_to_string(
    heading: &mut Vec<String>,
    regexes: &Regexes,
    captions: &[TableCaption<'_>],
    rows: &[TableRow<'_>],
) -> Result<String, WikiMarkupProcessingError> {
    let captions = table_captions_to_string(heading, captions, regexes)?;

    if let Some(rows) = table_rows_to_string(heading, rows, regexes)? {
        let table = if let Some(captions) = captions {
            if !captions.is_empty() {
                format!("\ncaption='{}'\n{}\n", captions, rows)
            } else {
                format!("\n{}\n", rows)
            }
        } else {
            format!("\n{}\n", rows)
        };

        Ok(table.to_string())
    } else {
        Ok(String::new())
    }
}

pub(super) fn table_captions_to_string(
    heading: &mut Vec<String>,
    table_captions: &[TableCaption<'_>],
    regexes: &Regexes,
) -> Result<Option<String>, WikiMarkupProcessingError> {
    if table_captions.is_empty() {
        Ok(None)
    } else {
        let mut documents = vec![];
        for tc in table_captions.iter() {
            documents.push(table_caption_to_string(heading, tc, regexes)?)
        }
        Ok(Some(documents.join("")))
    }
}

pub(super) fn table_rows_to_string(
    heading: &mut Vec<String>,
    table_rows: &[TableRow<'_>],
    regexes: &Regexes,
) -> Result<Option<String>, WikiMarkupProcessingError> {
    if table_rows.is_empty() {
        Ok(None)
    } else {
        let mut documents = vec![];
        for tr in table_rows.iter() {
            documents.push(table_row_to_string(heading, tr, regexes)?)
        }

        Ok(Some(documents.join("\n")))
    }
}

pub(super) fn table_cells_to_string(
    heading: &mut Vec<String>,
    table_cells: &[TableCell<'_>],
    regexes: &Regexes,
) -> Result<Option<String>, WikiMarkupProcessingError> {
    if table_cells.is_empty() {
        Ok(None)
    } else {
        let tag = match table_cells.first().unwrap().type_ {
            TableCellType::Heading => "||",
            TableCellType::Ordinary => "|",
        };

        let mut documents: Vec<String> = vec![];
        for tc in table_cells.iter() {
            let cell_text = table_cell_to_string(heading, tc, regexes)?;
            if cell_text.is_empty() {
                documents.push(" ".to_string())
            } else {
                documents.push(cell_text)
            }
        }

        Ok(Some(format!("{tag}{}{tag}", documents.join(tag))))
    }
}

pub(super) fn table_cell_to_string(
    heading: &mut Vec<String>,
    TableCell { content, .. }: &TableCell<'_>,
    regexes: &Regexes,
) -> ParseResult {
    let content = nodes_to_string(heading, content, regexes)?;
    let content = content.trim();
    if content.is_empty() {
        Ok(String::new())
    } else {
        Ok(content.to_string())
    }
}
pub(super) fn table_row_to_string(
    heading: &mut Vec<String>,
    TableRow { cells, .. }: &TableRow<'_>,
    regexes: &Regexes,
) -> ParseResult {
    let cells = table_cells_to_string(heading, cells, regexes)?;
    if let Some(cells) = cells {
        Ok(cells)
    } else {
        Ok(String::new())
    }
}
pub(super) fn table_caption_to_string(
    heading: &mut Vec<String>,
    TableCaption { content, .. }: &TableCaption<'_>,
    regexes: &Regexes,
) -> ParseResult {
    nodes_to_string(heading, content, regexes)
}

#[cfg(test)]
mod tests_table_cell_to_string {

    use parse_wiki_text::Node;

    use super::*;
    #[test]
    fn table_cell_to_string_ordinary_text() {
        let cell_attribute_text = "cell_attribute_text";
        let cell_content_text = "cell_content_text";
        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_content = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let input = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![cell_content],
            type_: TableCellType::Ordinary,
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction = table_cell_to_string(&mut vec![String::new()], &input, &regex).unwrap();
        assert_eq!(format!("{cell_content_text}"), extraction)
    }
    #[test]
    fn table_cell_to_string_heading_text() {
        let cell_attribute_text = "cell_attribute_text";
        let cell_content_text = "cell_content_text";
        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_content = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let input = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![cell_content],
            type_: TableCellType::Heading,
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction = table_cell_to_string(&mut vec![String::new()], &input, &regex).unwrap();
        assert_eq!(format!("{cell_content_text}"), extraction)
    }

    #[test]
    fn table_cells_to_string_ordinary_text() {
        let cell_attribute_text = "cell_attribute_text";
        let cell_content_text = "cell_content_text";
        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_attribute2 = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_content = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let cell_content2 = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let input = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![cell_content],
            type_: TableCellType::Ordinary,
            end: 0,
            start: 0,
        };
        let input2 = TableCell {
            attributes: Some(vec![cell_attribute2]),
            content: vec![cell_content2],
            type_: TableCellType::Ordinary,
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction = table_cells_to_string(&mut vec![String::new()], &[input, input2], &regex)
            .unwrap()
            .unwrap();
        assert_eq!(
            format!("|{cell_content_text}|{cell_content_text}|"),
            extraction
        )
    }
    #[test]
    fn table_cells_to_string_heading_text() {
        let cell_attribute_text = "cell_attribute_text";
        let cell_content_text = "cell_content_text";
        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_attribute2 = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_content = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let cell_content2 = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let input = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![cell_content],
            type_: TableCellType::Heading,
            end: 0,
            start: 0,
        };
        let input2 = TableCell {
            attributes: Some(vec![cell_attribute2]),
            content: vec![cell_content2],
            type_: TableCellType::Heading,
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction = table_cells_to_string(&mut vec![String::new()], &[input, input2], &regex)
            .unwrap()
            .unwrap();
        assert_eq!(
            format!("||{cell_content_text}||{cell_content_text}||"),
            extraction
        )
    }

    #[test]
    fn table_row_to_string_text_text() {
        let row_attribute_text = "row_attribute_text";
        let cell_attribute_text = "cell_attribute_text";
        let cell_content_text = "cell_content_text";
        let row_attribute = Node::Text {
            value: row_attribute_text,
            end: 0,
            start: 0,
        };

        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_content = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let row_cell = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![cell_content],
            type_: TableCellType::Heading,
            end: 0,
            start: 0,
        };

        let input = TableRow {
            attributes: vec![row_attribute],
            cells: vec![row_cell],
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction = table_row_to_string(&mut vec![String::new()], &input, &regex).unwrap();
        assert_eq!(format!("||{cell_content_text}||"), extraction)
    }

    #[test]
    fn table_rows_to_string_text_text() {
        let row_attribute_text = "row_attribute_text";
        let cell_attribute_text = "cell_attribute_text";
        let cell_content_text = "cell_content_text";
        let row_attribute = Node::Text {
            value: row_attribute_text,
            end: 0,
            start: 0,
        };

        let row_attribute2 = Node::Text {
            value: row_attribute_text,
            end: 0,
            start: 0,
        };

        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_attribute2 = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let cell_content = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let cell_content2 = Node::Text {
            value: cell_content_text,
            end: 0,
            start: 0,
        };
        let row_cell = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![cell_content],
            type_: TableCellType::Heading,
            end: 0,
            start: 0,
        };
        let row_cell2 = TableCell {
            attributes: Some(vec![cell_attribute2]),
            content: vec![cell_content2],
            type_: TableCellType::Ordinary,
            end: 0,
            start: 0,
        };
        let input = TableRow {
            attributes: vec![row_attribute],
            cells: vec![row_cell],
            end: 0,
            start: 0,
        };
        let input2 = TableRow {
            attributes: vec![row_attribute2],
            cells: vec![row_cell2],
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction = table_rows_to_string(&mut vec![String::new()], &[input, input2], &regex)
            .unwrap()
            .unwrap();
        assert_eq!(
            format!("||{cell_content_text}||\n|{cell_content_text}|"),
            extraction
        )
    }

    #[test]
    fn node_to_string_table_text() {
        let table_attribute_text = "table_attribute_text";
        let caption_attribute_text = "caption_attribute_text";
        let caption_content_text = "caption_content_text";
        let row_attribute_text = "row_attribute_text";
        let row_content_text = "row_content_text";
        let cell_attribute_text = "cell_attribute_text";

        let _table_attribute = Node::Text {
            value: table_attribute_text,
            end: 0,
            start: 0,
        };
        let caption_attribute = Node::Text {
            value: caption_attribute_text,
            end: 0,
            start: 0,
        };
        let caption_content = Node::Text {
            value: caption_content_text,
            end: 0,
            start: 0,
        };
        let row_attribute = Node::Text {
            value: row_attribute_text,
            end: 0,
            start: 0,
        };
        let row_content = Node::Text {
            value: row_content_text,
            end: 0,
            start: 0,
        };
        let cell_attribute = Node::Text {
            value: cell_attribute_text,
            end: 0,
            start: 0,
        };
        let caption = TableCaption {
            attributes: Some(vec![caption_attribute]),
            content: vec![caption_content],
            end: 0,
            start: 0,
        };
        let row_cell = TableCell {
            attributes: Some(vec![cell_attribute]),
            content: vec![row_content],
            type_: TableCellType::Ordinary,
            end: 0,
            start: 0,
        };
        let row = TableRow {
            attributes: vec![row_attribute],
            cells: vec![row_cell],
            end: 0,
            start: 0,
        };

        let regex = Regexes::new();

        let extraction =
            table_to_string(&mut vec![String::new()], &regex, &[caption], &[row]).unwrap();
        assert_eq!(
            format!("\ncaption='{caption_content_text}'\n|{row_content_text}|\n"),
            extraction
        )
    }
}
