use parse_wiki_text::*;

use super::{
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn unordered_list_items_to_string(
    list_items: &[ListItem<'_>],
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];

    for li in list_items.iter() {
        documents.push(format!(" - {}", list_item_to_string(li, regexes)?))
    }

    Ok(documents.join("\n"))
}

pub(super) fn ordered_list_items_to_string(
    list_items: &[ListItem<'_>],
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];

    for (c, li) in list_items.iter().enumerate() {
        documents.push(format!(" {c}. {}", list_item_to_string(li, regexes)?))
    }
    Ok(documents.join("\n"))
}

pub(super) fn list_item_to_string(
    ListItem { nodes, .. }: &ListItem<'_>,
    regexes: &Regexes,
) -> ParseResult {
    nodes_to_string(nodes, regexes)
}
