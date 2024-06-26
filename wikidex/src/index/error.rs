use face_api::apis::{crate_api::QueryError, Error};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
};

#[derive(Debug)]
pub enum IndexSearchError {
    QueryError(Error<QueryError>),
}

impl From<Error<QueryError>> for IndexSearchError {
    fn from(value: Error<QueryError>) -> Self {
        IndexSearchError::QueryError(value)
    }
}

impl StdError for IndexSearchError {}

impl Display for IndexSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IndexSearchError::QueryError(err) => {
                write!(f, "SearchService: {:?}", err)
            }
        }
    }
}
