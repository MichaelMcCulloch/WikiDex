use parse_wiki_text::Node;
use std::ops::Deref;

use super::{
    deflist::definition_list_items_to_string,
    listitems::{ordered_list_items_to_string, unordered_list_items_to_string},
    tables::table_to_string,
    template_params::refn_parameters_to_string,
    Regexes,
};
use crate::ingest::{pipeline::wikipedia::WikiMarkupProcessor, service::Process};

pub(crate) const STOP_PHRASES: [&str; 6] = [
    "References",
    "Bibliography",
    "See also",
    "Further reading",
    "External links",
    "Notes and references",
];
pub(crate) const HEADING_START: &str = "###HEADING_START###";
pub(crate) const HEADING_END: &str = "###HEADING_END###";

pub(crate) type ParseResult = Result<String, <WikiMarkupProcessor as Process>::E>;

pub(crate) fn process_to_article(nodes: &[Node<'_>], regexes: &Regexes) -> ParseResult {
    nodes_to_string(&mut vec![], nodes, regexes)
}

pub(super) fn nodes_to_string(
    heading: &mut Vec<String>,
    nodes: &[Node<'_>],
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];
    for n in nodes.iter() {
        if let Node::Heading {
            nodes: heading_nodes,
            ..
        } = n
        {
            let heading_name = nodes_to_string(heading, heading_nodes, regexes)?;

            if STOP_PHRASES.contains(&heading_name.as_str()) {
                break;
            }
        }

        documents.push(node_to_string(heading, n, regexes)?);
    }

    Ok(documents.join("").trim().to_string())
}

pub(super) fn node_to_string(
    heading: &mut Vec<String>,
    node: &Node<'_>,
    regexes: &Regexes,
) -> ParseResult {
    match node {
        Node::Bold { .. } => Ok(String::new()),
        Node::BoldItalic { .. } => Ok(String::new()),
        Node::Comment { .. } => Ok(String::new()),
        Node::HorizontalDivider { .. } => Ok(String::new()),
        Node::Italic { .. } => Ok(String::new()),
        Node::MagicWord { .. } => Ok(String::new()),
        Node::Category { .. } => Ok(String::new()),
        Node::Redirect { .. } => Ok(String::new()),
        Node::EndTag { .. } => Ok(String::new()),
        Node::Tag { .. } => Ok(String::new()),
        Node::Image { .. } => Ok(String::new()),
        Node::StartTag { .. } => Ok(String::new()),
        Node::ParagraphBreak { .. } => Ok(String::from("\n\n")),
        // Node::Heading {
        //     nodes,
        //     level: 1 | 2,
        //     ..
        // } => nodes_to_string(nodes, regexes).map(|heading| heading.to_string()),
        Node::Heading { nodes, level, .. } => {
            let new_heading = nodes_to_string(heading, nodes, regexes)?;
            // Calculate the level difference between the new level and the current length of headings
            // this always produces an empty first member in heading, not sure why.
            let heading_str = adjust_headings(level, heading, new_heading);
            Ok(heading_str)
        }
        Node::ExternalLink { nodes, .. } => {
            let document = nodes_to_string(heading, nodes, regexes)?;
            let str = document.deref().split(' ').collect::<Vec<_>>()[1..].join(" ");
            Ok(str)
        }
        Node::Preformatted { nodes, .. } => nodes_to_string(heading, nodes, regexes),

        Node::CharacterEntity { character, .. } => Ok(String::from(*character)),

        Node::Link { text, .. } => nodes_to_string(heading, text, regexes),
        Node::Parameter { default, name, .. } => {
            let name = nodes_to_string(heading, name, regexes)?;
            let default = match default {
                Some(default) => nodes_to_string(heading, default, regexes)?,
                None => String::new(),
            };
            Ok([name, default].join(": "))
        }

        Node::DefinitionList { items, .. } => {
            definition_list_items_to_string(heading, items, regexes)
        }
        Node::UnorderedList { items, .. } => {
            unordered_list_items_to_string(heading, items, regexes)
        }
        Node::OrderedList { items, .. } => ordered_list_items_to_string(heading, items, regexes),
        Node::Table { captions, rows, .. } => table_to_string(heading, regexes, captions, rows),
        Node::Template {
            name, parameters, ..
        } => {
            let name = nodes_to_string(heading, name, regexes)?;
            if regexes.refn.is_match(&name) || regexes.linktext.is_match(&name) {
                refn_parameters_to_string(heading, parameters, regexes)
            } else if regexes.language.is_match(&name) && !parameters.is_empty() {
                refn_parameters_to_string(heading, &parameters[1..], regexes)
            } else {
                Ok(String::new())
            }
        }
        Node::Text { value: "\n", .. } => Ok(String::new()),
        Node::Text { value, .. } => Ok(String::from(*value)),
    }
}

fn adjust_headings(level: &u8, heading: &mut Vec<String>, new_heading: String) -> String {
    let incr = *level as i8 - heading.len() as i8;

    // If level difference is positive, add placeholders for missing heading levels
    if incr > 0 {
        // Extend the heading vector with placeholders to reach the new level
        heading.extend(vec!["".to_string(); incr as usize]);
    }

    // If level difference is negative, remove excess heading levels
    if incr < 0 {
        // Pop the extra headings
        for _ in 0..(-incr) {
            heading.pop();
        }
    }

    // Replace the last heading with the new heading
    if let Some(last) = heading.last_mut() {
        *last = new_heading.to_string();
    } else {
        heading.push(new_heading.to_string());
    }

    // Construct the formatted heading string from the adjusted vector
    let heading_str = format!("{HEADING_START}{}{HEADING_END}", heading.join(":"));
    heading_str
}

#[cfg(test)]
mod tests_node_to_string {

    use parse_wiki_text::Configuration;

    use crate::{
        ingest::pipeline::wikipedia::{
            configurations::WIKIPEDIA_CONFIGURATION, markup_processor::parse::Regexes,
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

        let parse = configuration.parse(document_text).nodes;
        let regex = Regexes::new();

        let process = process_to_article(&parse, &regex).unwrap();
        println!("{process}")
    }
}
