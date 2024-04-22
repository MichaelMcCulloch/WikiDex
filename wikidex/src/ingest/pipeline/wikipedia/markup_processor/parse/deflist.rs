use parse_wiki_text::*;

use super::{
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn definition_list_item_type_to_string(
    definition_list_item_type: &DefinitionListItemType,
) -> ParseResult {
    match definition_list_item_type {
        DefinitionListItemType::Details => Ok(String::from("Details")),
        DefinitionListItemType::Term => Ok(String::from("Term")),
    }
}

pub(super) fn definition_list_item_to_string(
    DefinitionListItem { type_, nodes, .. }: &DefinitionListItem<'_>,
    regexes: &Regexes,
) -> ParseResult {
    let type_ = definition_list_item_type_to_string(type_)?;
    let nodes = nodes_to_string(nodes, regexes)?;
    Ok([type_, nodes].join(""))
}

pub(super) fn definition_list_items_to_string(
    definition_list_items: &[DefinitionListItem<'_>],
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];
    for dli in definition_list_items.iter() {
        documents.push(definition_list_item_to_string(dli, regexes)?)
    }
    Ok(documents.join("\n"))
}
