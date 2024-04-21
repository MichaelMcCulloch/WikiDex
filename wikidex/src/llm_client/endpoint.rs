use std::{error::Error, fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ModelEndpoint {
    Triton,
    OpenAi,
}

impl Display for ModelEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelEndpoint::Triton => write!(f, "Triton"),
            ModelEndpoint::OpenAi => write!(f, "Openai"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseModelEndpointError;
impl Error for ParseModelEndpointError {}
impl Display for ParseModelEndpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to parse model kind. Must be one of [Triton, OpenAi]"
        )
    }
}
impl FromStr for ModelEndpoint {
    type Err = ParseModelEndpointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            "triton" => Ok(ModelEndpoint::Triton),
            "openai" => Ok(ModelEndpoint::OpenAi),
            _ => Err(ParseModelEndpointError),
        }
    }
}
