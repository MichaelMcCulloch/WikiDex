use parse_wiki_text::Parameter;

use crate::{
    ingest::wikipedia::{
        helper::wiki::UnlabledDocument, markup_processor::Process, WikiMarkupProcessor,
    },
    llm::SyncOpenAiService,
};

use super::{nodes::nodes_to_string, Regexes};

pub(super) async fn template_parameters_to_string(
    parameters: &[Parameter<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for p in parameters.iter() {
        documents.push(template_parameter_to_string(p, regexes, client).await?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}

pub(super) async fn refn_parameters_to_string(
    parameters: &[Parameter<'_>],
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let mut documents = vec![];
    for p in parameters.iter() {
        documents.push(refn_parameter_to_string(p, regexes, client).await?)
    }
    Ok(UnlabledDocument::join_all(documents, &""))
}
pub(super) async fn refn_parameter_to_string(
    Parameter { value, .. }: &Parameter<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    nodes_to_string(value, regexes, client).await
}
pub(super) async fn template_parameter_to_string(
    Parameter { name, value, .. }: &Parameter<'_>,
    regexes: &Regexes,
    client: &SyncOpenAiService,
) -> Result<UnlabledDocument, <WikiMarkupProcessor as Process>::E> {
    let value = nodes_to_string(value, regexes, client).await?;
    let name = match name {
        Some(name) => nodes_to_string(name, regexes, client).await?,
        None => UnlabledDocument::new(),
    };
    Ok(UnlabledDocument::join_all(vec![name, value], ": "))
}
