mod error;

use backoff::{future::retry, ExponentialBackoff};
use fbthrift_transport::{AsyncTransport, AsyncTransportConfiguration};

use nebula_client::v3::{GraphClient, GraphQuery as _, GraphTransportResponseHandler};

use url::Url;

use self::error::PlainTextProcessingError;

use super::service::Process;

#[derive(Clone)]
pub(crate) struct PlainTextProcessor;
impl PlainTextProcessor {
    pub(crate) async fn new(
        url: Url,
        username: &str,
        password: &str,
    ) -> Result<Self, <PlainTextProcessor as Process>::E> {
        let mut session = retry(ExponentialBackoff::default(), || async {
            let addr = format!("{}:{}", url.domain().unwrap(), url.port().unwrap());
            // let _addr = SocketAddrV4::new(Ipv4Addr::new(192u8, 168u8, 1u8, 120u8), 9669);
            // let _addr = url.unwrap().to_string();
            log::info!("{addr}");
            let transport = AsyncTransport::with_tokio_tcp_connect(
                addr,
                AsyncTransportConfiguration::new(GraphTransportResponseHandler),
            )
            .await
            .map_err(PlainTextProcessingError::Io);
            let transport = match transport {
                Ok(transport) => Ok(transport),
                Err(e) => {
                    log::info!("{e}");
                    Err(e)
                }
            }?;

            let client = GraphClient::new(transport);

            let graph_session = client
                .authenticate(&username.as_bytes().to_vec(), &password.as_bytes().to_vec())
                .await
                .map_err(PlainTextProcessingError::NebulaAuthentication);

            let graph_session = match graph_session {
                Ok(graph_session) => Ok(graph_session),
                Err(e) => {
                    log::info!("{e}");
                    Err(e)
                }
            }?;
            Ok(graph_session)
        })
        .await?;

        let res = session.show_hosts().await?;
        println!("{res:?}");

        Ok(Self)
    }
}
impl Process for PlainTextProcessor {
    type E = PlainTextProcessingError;
    fn process(&self, _text: &str) -> Result<String, Self::E> {
        todo!()
    }
}
