use parse_wiki_text::Node;
use std::ops::Deref;

use super::{
    deflist::definition_list_items_to_string,
    listitems::{ordered_list_items_to_string, unordered_list_items_to_string},
    tables::table_to_string,
    template_params::refn_parameters_to_string,
    Regexes,
};
use crate::{
    ingest::wikipedia::{markup_processor::Process, WikiMarkupProcessor},
    llm::SyncOpenAiService,
};

pub(crate) const STOP_PHRASES: [&str; 5] = [
    "References",
    "Bibliography",
    "See also",
    "Further reading",
    "External links",
];

pub(crate) type ParseResult = Result<String, <WikiMarkupProcessor as Process>::E>;

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
                if STOP_PHRASES.contains(&heading_name.as_str()) {
                    break;
                } else {
                    documents.push(node_to_string(n, regexes, client)?)
                }
            }
            _ => documents.push(node_to_string(n, regexes, client)?),
        }
    }
    Ok(documents.join(""))
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
        | Node::StartTag { .. } => Ok(String::new()),
        Node::ParagraphBreak { .. } | Node::Heading { .. } => Ok(String::from("\n\n")),

        Node::ExternalLink { nodes, .. } => {
            let document = nodes_to_string(nodes, regexes, client)?;
            let str = document.deref().split(' ').collect::<Vec<_>>()[1..].join(" ");
            Ok(str)
        }
        Node::Preformatted { nodes, .. } => nodes_to_string(nodes, regexes, client),

        Node::CharacterEntity { character, .. } => Ok(String::from(*character)),

        Node::Link { text, .. } => nodes_to_string(text, regexes, client),
        Node::Parameter { default, name, .. } => {
            let name = nodes_to_string(name, regexes, client)?;
            let default = match default {
                Some(default) => nodes_to_string(default, regexes, client)?,
                None => String::new(),
            };
            Ok(vec![name, default].join(": "))
        }

        Node::DefinitionList { items, .. } => {
            definition_list_items_to_string(items, regexes, client)
        }
        Node::UnorderedList { items, .. } => unordered_list_items_to_string(items, regexes, client),
        Node::OrderedList { items, .. } => ordered_list_items_to_string(items, regexes, client),
        Node::Table { captions, rows, .. } => table_to_string(captions, regexes, client, rows),
        Node::Template {
            name, parameters, ..
        } => {
            let name = nodes_to_string(name, regexes, client)?;
            if regexes.refn.is_match(&name) || regexes.linktext.is_match(&name) {
                refn_parameters_to_string(&parameters, regexes, client)
            } else if regexes.language.is_match(&name) && !parameters.is_empty() {
                refn_parameters_to_string(&parameters[1..], regexes, client)
            } else {
                Ok(String::new())
            }
        }
        Node::Text { value: "\n", .. } => Ok(String::new()),
        Node::Text { value, .. } => Ok(String::from(*value)),
    }
}
