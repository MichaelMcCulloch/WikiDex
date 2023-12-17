use parse_wiki_text::Node;
use std::ops::Deref;

use super::{
    deflist::definition_list_items_to_string,
    listitems::{ordered_list_items_to_string, unordered_list_items_to_string},
    tables::table_to_string,
    template_params::refn_parameters_to_string,
    Regexes,
};
use crate::ingest::wikipedia::{markup_processor::Process, WikiMarkupProcessor};

pub(crate) const STOP_PHRASES: [&str; 6] = [
    "References",
    "Bibliography",
    "See also",
    "Further reading",
    "External links",
    "Notes and references",
];

pub(crate) type ParseResult = Result<String, <WikiMarkupProcessor as Process>::E>;

pub(crate) fn process_to_article(nodes: &[Node<'_>], regexes: &Regexes) -> ParseResult {
    let output = nodes_to_string(nodes, regexes)?;

    let output = regexes
        .twospace
        .split(&output)
        .collect::<Vec<_>>()
        .join(" ");
    let output = regexes
        .space_coma
        .split(&output)
        .collect::<Vec<_>>()
        .join(",");
    let output = regexes
        .space_period
        .split(&output)
        .collect::<Vec<_>>()
        .join(".");
    let output = regexes
        .pilcrow
        .split(&output)
        .collect::<Vec<_>>()
        .join("\n");
    let output = regexes
        .threelines
        .split(&output)
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok(output)
}

pub(super) fn nodes_to_string(nodes: &[Node<'_>], regexes: &Regexes) -> ParseResult {
    let mut documents = vec![];
    for n in nodes.iter() {
        match n {
            Node::Heading {
                nodes: heading_nodes,
                ..
            } => {
                let heading_name = nodes_to_string(heading_nodes, regexes)?;
                if STOP_PHRASES.contains(&heading_name.as_str()) {
                    break;
                } else {
                    documents.push(node_to_string(n, regexes)?)
                }
            }
            _ => documents.push(node_to_string(n, regexes)?),
        }
    }
    Ok(documents.join(" ").trim().to_string())
}

pub(super) fn node_to_string(node: &Node<'_>, regexes: &Regexes) -> ParseResult {
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
        Node::ParagraphBreak { .. } => Ok(String::from("¶")), //fight me
        Node::Heading {
            nodes,
            level: 1 | 2,
            ..
        } => nodes_to_string(nodes, regexes).map(|heading| format!("\n\n{heading}")),
        Node::Heading { nodes, .. } => {
            nodes_to_string(nodes, regexes).map(|heading| format!("\n{heading}\n"))
        }

        Node::ExternalLink { nodes, .. } => {
            let document = nodes_to_string(nodes, regexes)?;
            let str = document.deref().split(' ').collect::<Vec<_>>()[1..].join(" ");
            Ok(str)
        }
        Node::Preformatted { nodes, .. } => nodes_to_string(nodes, regexes),

        Node::CharacterEntity { character, .. } => Ok(String::from(*character)),

        Node::Link { text, .. } => nodes_to_string(text, regexes),
        Node::Parameter { default, name, .. } => {
            let name = nodes_to_string(name, regexes)?;
            let default = match default {
                Some(default) => nodes_to_string(default, regexes)?,
                None => String::new(),
            };
            Ok([name, default].join(": "))
        }

        Node::DefinitionList { items, .. } => definition_list_items_to_string(items, regexes),
        Node::UnorderedList { items, .. } => unordered_list_items_to_string(items, regexes),
        Node::OrderedList { items, .. } => ordered_list_items_to_string(items, regexes),
        Node::Table { captions, rows, .. } => table_to_string(regexes, captions, rows),
        Node::Template {
            name, parameters, ..
        } => {
            let name = nodes_to_string(name, regexes)?;
            if regexes.refn.is_match(&name) || regexes.linktext.is_match(&name) {
                refn_parameters_to_string(parameters, regexes)
            } else if regexes.language.is_match(&name) && !parameters.is_empty() {
                refn_parameters_to_string(&parameters[1..], regexes)
            } else {
                Ok(String::new())
            }
        }
        Node::Text { value: "\n", .. } => Ok(String::new()),
        Node::Text { value, .. } => Ok(String::from(*value)),
    }
}

#[cfg(test)]
mod tests_node_to_string {

    use parse_wiki_text::Configuration;

    use crate::{
        ingest::wikipedia::{
            configurations::WIKIPEDIA_CONFIGURATION, helper::text::RecursiveCharacterTextSplitter,
            markup_processor::parse::Regexes,
        },
        test_data::SUPREME_COURT_VOL_129,
    };

    use super::process_to_article;

    #[test]
    fn read_document_file_to_string() {
        std::env::set_var("RUST_LOG", "info");
        env_logger::init();
        let configuration = Configuration::new(WIKIPEDIA_CONFIGURATION);

        let document_text = SUPREME_COURT_VOL_129;

        let parse = configuration.parse(&document_text).nodes;
        let regex = Regexes::new();

        let process = process_to_article(&parse, &regex).unwrap();
        println!("{process}")
    }
}
