use redis::{FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value};
use rkyv::{archived_root, Archive, Deserialize, Infallible, Serialize};

use crate::formatter::{Provenance, TextFormatter};
#[derive(Clone, Serialize, Deserialize, Archive, Debug)]
pub(crate) struct Document {
    pub(crate) index: i64,
    pub(crate) text: String,
    pub(crate) provenance: Provenance,
}

impl FromRedisValue for Document {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(bytes) = v {
            let archived = unsafe { archived_root::<Document>(bytes.as_slice()) };
            archived.deserialize(&mut Infallible).map_err(|_| {
                RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed"))
            })
        } else {
            Err(RedisError::from((
                redis::ErrorKind::TypeError,
                "Expected bytes, got something else",
            )))
        }
    }
}

impl ToRedisArgs for Document {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = rkyv::to_bytes::<_, 2048>(self).unwrap();
        out.write_arg(&bytes);
    }
}

impl TextFormatter for Document {
    fn format_document(&self) -> String {
        format!("#{}: {}\n", self.index, self.text)
    }
}
