use parse_wiki_text::Node;
use std::ops::Deref;

use super::{
    super::WikiMarkupProcessingError::LlmError,
    deflist::definition_list_items_to_string,
    listitems::{ordered_list_items_to_string, unordered_list_items_to_string},
    llm::process_table_to_llm,
    tables::{table_captions_to_string, table_rows_to_string},
    template_params::refn_parameters_to_string,
    Regexes,
};
use crate::{
    ingest::wikipedia::{
        helper::wiki::UnlabledDocument, markup_processor::Process, WikiMarkupProcessor,
    },
    llm::SyncOpenAiService,
};

pub(crate) const STOP_PHRASES: [&str; 5] = [
    "References",
    "Bibliography",
    "See also",
    "Further reading",
    "External links",
];
pub(crate) async fn process_to_article(
    nodes: &[Node<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    nodes_to_string(&nodes, regexes, client).await
}

#[async_recursion::async_recursion]
pub(super) async fn nodes_to_string(
    nodes: &[Node<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for n in nodes.iter() {
        match n {
            Node::Heading {
                nodes: heading_nodes,
                ..
            } => {
                let heading_name = nodes_to_string(heading_nodes, regexes, client).await?;
                if STOP_PHRASES.contains(&heading_name.document.as_str()) {
                    break;
                } else {
                    documents.push(node_to_string(n, regexes, client).await?)
                }
            }
            _ => documents.push(node_to_string(n, regexes, client).await?),
        }
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) async fn node_to_string(
    node: &Node<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    match node {
        Node::Bold { .. }
        | Node::BoldItalic { .. }
        | Node::Comment { .. }
        | Node::HorizontalDivider { .. }
        | Node::Italic { .. }
        | Node::MagicWord { .. }
        | Node::Category { .. }
        | Node::Redirect { .. }
        | Node::EndTag { .. }
        | Node::Tag { .. }
        | Node::Image { .. }
        | Node::StartTag { .. } => Ok(UnlabledDocument::new()),
        Node::ParagraphBreak { .. } | Node::Heading { .. } => {
            Ok(UnlabledDocument::from_str(String::from("\n\n")))
        }

        Node::ExternalLink { nodes, .. } => {
            let document = nodes_to_string(nodes, regexes, client).await?;
            let str = document.document.deref().split(' ').collect::<Vec<_>>()[1..].join(" ");
            Ok(UnlabledDocument::from_str_and_vec(str, document.table))
        }
        Node::Preformatted { nodes, .. } => nodes_to_string(nodes, regexes, client).await,

        Node::CharacterEntity { character, .. } => {
            Ok(UnlabledDocument::from_str(String::from(*character)))
        }

        Node::Link { text, .. } => nodes_to_string(text, regexes, client).await,
        Node::Parameter { default, name, .. } => {
            let name = nodes_to_string(name, regexes, client).await?;
            let default = match default {
                Some(default) => nodes_to_string(default, regexes, client).await?,
                None => UnlabledDocument::new(),
            };
            Ok(UnlabledDocument::join_all(vec![name, default], ": "))
        }

        Node::DefinitionList { items, .. } => {
            definition_list_items_to_string(items, regexes, client).await
        }
        Node::UnorderedList { items, .. } => {
            unordered_list_items_to_string(items, regexes, client).await
        }
        Node::OrderedList { items, .. } => {
            ordered_list_items_to_string(items, regexes, client).await
        }
        // Node::Table { .. } => String::new(),
        Node::Table { captions, rows, .. } => {
            let captions = table_captions_to_string(captions, regexes, client).await?;
            let rows = table_rows_to_string(rows, regexes, client).await?;
            let table = if captions.document.is_empty() {
                format!("\n<table>\n{}</table>\n", rows.document)
            } else {
                format!(
                    "\n<table caption='{}'>\n{}</table>\n",
                    captions.document, rows.document
                )
            };
            process_table_to_llm(&table, client).await.map_err(LlmError)
        }
        Node::Template {
            name, parameters, ..
        } => {
            let name = nodes_to_string(name, regexes, client).await?;
            if regexes.refn.is_match(&name.document) || regexes.linktext.is_match(&name.document) {
                refn_parameters_to_string(&parameters, regexes, client).await
            } else if regexes.language.is_match(&name.document) && !parameters.is_empty() {
                refn_parameters_to_string(&parameters[1..], regexes, client).await
            } else {
                Ok(UnlabledDocument::new())
            }
        }
        Node::Text { value: "\n", .. } => Ok(UnlabledDocument::new()),
        Node::Text { value, .. } => Ok(UnlabledDocument::from_str(String::from(*value))),
    }
}
