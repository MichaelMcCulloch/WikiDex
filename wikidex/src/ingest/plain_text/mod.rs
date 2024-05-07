mod error;

use std::{net::ToSocketAddrs, sync::Arc};

use async_compat::Compat;

use backoff::{future::retry, ExponentialBackoff};
use fbthrift_transport::{AsyncTransport, AsyncTransportConfiguration};

use indicatif::MultiProgress;
use nebula_client::v3::{GraphClient, GraphSession as GS, GraphTransportResponseHandler};

use crate::llm_client::LlmClientImpl;
use tokio::{net::TcpStream, time::Sleep};
use url::Url;

use crate::embedding_client::EmbeddingClient;

use self::error::PlainTextProcessingError;

use super::service::Process;

pub(crate) type GraphSession =
    GS<AsyncTransport<Compat<TcpStream>, Sleep, GraphTransportResponseHandler>>;

pub(crate) struct PlainTextProcessor {
    pub(crate) graph: GraphSession,
    pub(crate) llm: Arc<LlmClientImpl>,
    pub(crate) embed: Arc<EmbeddingClient>,
}
impl PlainTextProcessor {
    pub(crate) fn new(
        llm: Arc<LlmClientImpl>,
        embed: Arc<EmbeddingClient>,
        graph: GraphSession,
        _multi_progress: MultiProgress,
    ) -> Self {
        // let res = graph_session
        //     .show_hosts()
        //     .await
        //     .map_err(PlainTextProcessingError::GraphQueryError)?;

        // log::info!("{res:?}");

        Self { graph, llm, embed }
    }
}
impl Process for PlainTextProcessor {
    type E = PlainTextProcessingError;
    fn process(&self, _text: &str) -> Result<String, Self::E> {
        todo!()
    }
}
pub(crate) async fn graph_client(
    url: Url,
    username: &str,
    password: &str,
) -> Result<GraphSession, <PlainTextProcessor as Process>::E> {
    let graph_session = retry(ExponentialBackoff::default(), || async {
        // let addr = format!("{}:{}", url.domain().unwrap(), url.port().unwrap());
        let addr = url
            .domain()
            .and_then(|domain| domain.to_socket_addrs().ok())
            .and_then(|mut list| list.next())
            .ok_or(PlainTextProcessingError::MalformedAddress)?;
        let transport = AsyncTransport::with_tokio_tcp_connect(
            addr,
            AsyncTransportConfiguration::new(GraphTransportResponseHandler),
        )
        .await
        .map_err(PlainTextProcessingError::Io)?;

        let client = GraphClient::new(transport);

        let graph_session = client
            .authenticate(&username.as_bytes().to_vec(), &password.as_bytes().to_vec())
            .await
            .map_err(PlainTextProcessingError::NebulaAuthentication)?;

        Ok(graph_session)
    })
    .await?;

    Ok(graph_session)
}
