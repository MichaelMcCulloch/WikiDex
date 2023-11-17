use parse_wiki_text::*;

use crate::{
    ingest::wikipedia::{
        helper::wiki::UnlabledDocument, markup_processor::Process, WikiMarkupProcessor,
    },
    llm::OpenAiService,
};

use super::{nodes::nodes_to_string, Regexes};

pub(super) async fn unordered_list_items_to_string(
    list_items: &Vec<ListItem<'_>>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for li in list_items.iter() {
        let document = list_item_to_string(&li, regexes, client).await?;
        let document = document.prepend(" - ");
        documents.push(document)
    }
    Ok(UnlabledDocument::join_all(documents, &"\n"))
}

pub(super) async fn ordered_list_items_to_string(
    list_items: &Vec<ListItem<'_>>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for (c, li) in list_items.iter().enumerate() {
        let document = list_item_to_string(&li, regexes, client).await?;
        let document = document.prepend(format!(" {c}. ").as_str());
        documents.push(document)
    }
    Ok(UnlabledDocument::join_all(documents, &"\n"))
}

pub(super) async fn list_item_to_string(
    ListItem { nodes, .. }: &ListItem<'_>,
    regexes: &Regexes,
    client: &OpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    nodes_to_string(nodes, regexes, client).await
}
