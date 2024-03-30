use crate::index::FaissIndex;

pub(crate) struct IndexEngine {
    index: FaissIndex,
}

impl IndexEngine {
    pub(crate) fn new(index: FaissIndex) -> Self {
        Self { index }
    }
}
