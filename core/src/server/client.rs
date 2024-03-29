use bytes::Bytes;
use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
pub struct Client(UnboundedReceiver<Bytes>);

impl Client {
    pub(crate) fn new() -> (Self, UnboundedSender<Bytes>) {
        let (tx, rx) = unbounded_channel();
        (Self(rx), tx)
    }
}

impl Stream for Client {
    type Item = Result<Bytes, actix_web::http::Error>;
    /// This does NOT work without self.0 being a tokio receiver of some kind
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
