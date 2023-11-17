use parse_wiki_text::*;

use crate::{ingest::wikipedia::helper::wiki::UnlabledDocument, llm::OpenAiService};

use super::{nodes::nodes_to_string, regexes::Regexes, Process, WikiMarkupProcessor};

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

pub(super) async fn definition_list_item_to_string(
    DefinitionListItem { type_, nodes, .. }: &DefinitionListItem<'_>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let type_ = definition_list_item_type_to_string(type_)?;
    let nodes = nodes_to_string(nodes, regexes, client).await?;
    Ok(UnlabledDocument::join_all(vec![type_, nodes], &""))
}

pub(super) async fn definition_list_items_to_string(
    definition_list_items: &Vec<DefinitionListItem<'_>>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut str = vec![];
    for dli in definition_list_items.iter() {
        str.push(definition_list_item_to_string(&dli, regexes, client).await?)
    }
    Ok(UnlabledDocument::join_all(str, &"\n"))
}
