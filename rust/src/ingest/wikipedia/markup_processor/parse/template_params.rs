use parse_wiki_text::Parameter;

use crate::{ingest::wikipedia::helper::wiki::UnlabledDocument, llm::SyncOpenAiService};

use super::{
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn _template_parameters_to_string(
    parameters: &[Parameter<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let mut documents = vec![];
    for p in parameters.iter() {
        documents.push(_template_parameter_to_string(p, regexes, client)?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) fn refn_parameters_to_string(
    parameters: &[Parameter<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let mut documents = vec![];
    for p in parameters.iter() {
        documents.push(refn_parameter_to_string(p, regexes, client)?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}
pub(super) fn refn_parameter_to_string(
    Parameter { value, .. }: &Parameter<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    nodes_to_string(value, regexes, client)
}
pub(super) fn _template_parameter_to_string(
    Parameter { name, value, .. }: &Parameter<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> ParseResult {
    let value = nodes_to_string(value, regexes, client)?;
    let name = match name {
        Some(name) => nodes_to_string(name, regexes, client)?,
        None => UnlabledDocument::new(),
    };
    Ok(UnlabledDocument::join_all(vec![name, value], ": "))
}
