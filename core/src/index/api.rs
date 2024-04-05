use face_api::{
    apis::{configuration::Configuration, crate_api as face},
    models::Query as FaceQuery,
};

use reqwest::Client;
use url::Url;

use super::{IndexSearchError, SearchService};

pub(crate) struct FaceIndex {
    configuration: Configuration,
}

impl FaceIndex {
    pub fn new(url: Url) -> Self {
        let url = match url.as_str().strip_suffix('/') {
            Some(url_safe) => url_safe,
            None => url.as_str(),
        };
        let configuration = Configuration {
            base_path: url.to_string(),
            user_agent: Some("Oracle-Core/0.1.0/rust".to_owned()),
            client: Client::new(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        };
        Self { configuration }
    }
}

impl SearchService for FaceIndex {
    type E = IndexSearchError;

    async fn search(&self, query: Vec<f32>, neighbors: usize) -> Result<Vec<i64>, Self::E> {
        let request = FaceQuery::new(neighbors as i32, query);
        let response = face::query(&self.configuration, request)
            .await
            .map_err(IndexSearchError::QueryError)?;
        Ok(response.neighbors)
    }
}
