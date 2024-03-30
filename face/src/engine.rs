use std::fmt::{Display, Formatter};

use actix_web::rt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc::channel, Mutex};

use crate::index::FaissIndex;
use crate::index::IndexError;
use crate::index::IndexSearchError;
use crate::index::SearchService;

use nolock::queues::{
    mpsc::jiffy::{self, AsyncSender},
    EnqueueError,
};
use tokio::sync::mpsc::Sender;

struct IndexArguments {
    embedding: Vec<f32>,
    neighbors: usize,
    sender: Sender<Vec<i64>>,
}

pub(crate) struct IndexEngine {
    index_queue: AsyncSender<IndexArguments>,
}

#[derive(Debug)]
pub(crate) enum IndexEngineError {
    QueueError(EnqueueError),
    IndexSearchError(IndexSearchError),
    IndexError(IndexError),
    SendError(SendError<Vec<i64>>),
    NoNeighbors,
}

impl std::error::Error for IndexEngineError {}

impl Display for IndexEngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexEngineError::QueueError(e) => write!(f, "{e:?}"),
            IndexEngineError::NoNeighbors => write!(f, "IndexEngine: NoNeighbors"),
            IndexEngineError::IndexSearchError(_) => todo!(),
            IndexEngineError::IndexError(_) => todo!(),
            IndexEngineError::SendError(_) => todo!(),
        }
    }
}

impl IndexEngine {
    pub(crate) async fn new(index: FaissIndex) -> Self {
        let mutex = Mutex::new(index);

        let (mut rx, tx) = jiffy::async_queue::<IndexArguments>();
        rt::spawn(async move {
            while let Ok(arguments) = rx.dequeue().await {
                let neighbors = mutex
                    .lock()
                    .await
                    .search(&arguments.embedding, arguments.neighbors)
                    .map_err(IndexEngineError::IndexSearchError)?;
                arguments
                    .sender
                    .send(neighbors)
                    .await
                    .map_err(IndexEngineError::SendError)?
            }
            Ok::<(), IndexEngineError>(())
        });
        Self { index_queue: tx }
    }
}

impl IndexEngine {
    pub(crate) async fn query(
        &self,
        embedding: Vec<f32>,
        neighbors: usize,
    ) -> Result<Vec<i64>, IndexEngineError> {
        let (sender, mut rx) = channel(1);
        let index_arguments = IndexArguments {
            embedding,
            neighbors,
            sender,
        };
        self.index_queue
            .enqueue(index_arguments)
            .map_err(|(_, e)| IndexEngineError::QueueError(e))?;

        match rx.recv().await {
            Some(neighbors) => Ok(neighbors),
            None => Err(IndexEngineError::NoNeighbors),
        }
    }
}
