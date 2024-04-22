use parse_wiki_text::Parameter;

use super::{
    nodes::{nodes_to_string, ParseResult},
    Regexes,
};

pub(super) fn _template_parameters_to_string(
    heading: (usize, &str),
    parameters: &[Parameter<'_>],
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];
    for p in parameters.iter() {
        documents.push(_template_parameter_to_string(heading, p, regexes)?)
    }
    Ok(documents.join(""))
}

pub(super) fn refn_parameters_to_string(
    heading: (usize, &str),
    parameters: &[Parameter<'_>],
    regexes: &Regexes,
) -> ParseResult {
    let mut documents = vec![];
    for p in parameters.iter() {
        documents.push(refn_parameter_to_string(heading, p, regexes)?)
    }
    Ok(documents.join(""))
}
pub(super) fn refn_parameter_to_string(
    heading: (usize, &str),
    Parameter { value, .. }: &Parameter<'_>,
    regexes: &Regexes,
) -> ParseResult {
    nodes_to_string(heading, value, regexes)
}
pub(super) fn _template_parameter_to_string(
    heading: (usize, &str),
    Parameter { name, value, .. }: &Parameter<'_>,
    regexes: &Regexes,
) -> ParseResult {
    let value = nodes_to_string(heading, value, regexes)?;
    let name = match name {
        Some(name) => nodes_to_string(heading, name, regexes)?,
        None => String::new(),
    };
    Ok([name, value].join(": "))
}
