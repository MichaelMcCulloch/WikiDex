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
        helper::wiki::{DescribedTable, UnlabledDocument},
        markup_processor::Process,
        WikiMarkupProcessor,
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

pub(crate) type ParseResult = Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E>;

pub(crate) fn process_to_article(
    nodes: &[Node<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    nodes_to_string(&nodes, regexes, client)
}

pub(super) fn nodes_to_string(
    nodes: &[Node<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let mut documents = vec![];
    for n in nodes.iter() {
        match n {
            Node::Heading {
                nodes: heading_nodes,
                ..
            } => {
                let heading_name = nodes_to_string(heading_nodes, regexes, client)?;
                if STOP_PHRASES.contains(&heading_name.document.as_str()) {
                    break;
                } else {
                    documents.push(node_to_string(n, regexes, client)?)
                }
            }
            _ => documents.push(node_to_string(n, regexes, client)?),
        }
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) fn node_to_string(
    node: &Node<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
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
            let document = nodes_to_string(nodes, regexes, client)?;
            let str = document.document.deref().split(' ').collect::<Vec<_>>()[1..].join(" ");
            Ok(UnlabledDocument::from_str_and_vec(str, document.table))
        }
        Node::Preformatted { nodes, .. } => nodes_to_string(nodes, regexes, client),

        Node::CharacterEntity { character, .. } => {
            Ok(UnlabledDocument::from_str(String::from(*character)))
        }

        Node::Link { text, .. } => nodes_to_string(text, regexes, client),
        Node::Parameter { default, name, .. } => {
            let name = nodes_to_string(name, regexes, client)?;
            let default = match default {
                Some(default) => nodes_to_string(default, regexes, client)?,
                None => UnlabledDocument::new(),
            };
            Ok(UnlabledDocument::join_all(vec![name, default], ": "))
        }

        Node::DefinitionList { items, .. } => {
            definition_list_items_to_string(items, regexes, client)
        }
        Node::UnorderedList { items, .. } => unordered_list_items_to_string(items, regexes, client),
        Node::OrderedList { items, .. } => ordered_list_items_to_string(items, regexes, client),
        // Node::Table { .. } => String::new(),
        Node::Table { captions, rows, .. } => {
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
                let description =
                    process_table_to_llm(&table_for_summary, client).map_err(LlmError)?;
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
        Node::Template {
            name, parameters, ..
        } => {
            let name = nodes_to_string(name, regexes, client)?;
            if regexes.refn.is_match(&name.document) || regexes.linktext.is_match(&name.document) {
                refn_parameters_to_string(&parameters, regexes, client)
            } else if regexes.language.is_match(&name.document) && !parameters.is_empty() {
                refn_parameters_to_string(&parameters[1..], regexes, client)
            } else {
                Ok(UnlabledDocument::new())
            }
        }
        Node::Text { value: "\n", .. } => Ok(UnlabledDocument::new()),
        Node::Text { value, .. } => Ok(UnlabledDocument::from_str(String::from(*value))),
    }
}
