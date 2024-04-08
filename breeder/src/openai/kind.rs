use std::{error::Error, fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ModelKind {
    Instruct,
    Chat,
}

#[derive(Debug)]
pub(crate) struct ParseModelKindError;
impl Error for ParseModelKindError {}
impl Display for ParseModelKindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to parse model kind. Must be one of [instruct, chat]/"
        )
    }
}
impl FromStr for ModelKind {
    type Err = ParseModelKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            "instruct" => Ok(ModelKind::Instruct),
            "chat" => Ok(ModelKind::Chat),
            _ => Err(ParseModelKindError),
        }
    }
}
