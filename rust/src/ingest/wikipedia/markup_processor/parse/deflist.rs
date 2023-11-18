use parse_wiki_text::*;

use crate::{
    ingest::wikipedia::{
        helper::wiki::UnlabledDocument, markup_processor::Process, WikiMarkupProcessor,
    },
    llm::SyncOpenAiService,
};

use super::{nodes::nodes_to_string, Regexes};

pub(super) fn definition_list_item_type_to_string(
    definition_list_item_type: &DefinitionListItemType,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    match definition_list_item_type {
        DefinitionListItemType::Details => Ok(UnlabledDocument {
            document: String::from("Details"),
            table: vec![],
        }),
        DefinitionListItemType::Term => Ok(UnlabledDocument {
            document: String::from("Term"),
            table: vec![],
        }),
    }
}

pub(super) fn definition_list_item_to_string(
    DefinitionListItem { type_, nodes, .. }: &DefinitionListItem<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let type_ = definition_list_item_type_to_string(type_)?;
    let nodes = nodes_to_string(nodes, regexes, client)?;
    Ok(UnlabledDocument::join_all(vec![type_, nodes], &""))
}

pub(super) fn definition_list_items_to_string(
    definition_list_items: &Vec<DefinitionListItem<'_>>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut str = vec![];
    for dli in definition_list_items.iter() {
        str.push(definition_list_item_to_string(&dli, regexes, client)?)
    }
    Ok(UnlabledDocument::join_all(str, &"\n"))
}
