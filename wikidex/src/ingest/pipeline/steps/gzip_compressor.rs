use flate2::{read::GzDecoder, write::GzEncoder};

use std::io::{self, Read, Write};

use crate::ingest::pipeline::{
    document::{DocumentCompressed, DocumentHeading},
    error::{CompressionError, PipelineError},
};

use super::PipelineStep;

pub(crate) struct Compressor;

fn compress_text(text: &str) -> Result<Vec<u8>, io::Error> {
    let mut text_compress = vec![];
    {
        let mut encoder = GzEncoder::new(&mut text_compress, flate2::Compression::new(9));
        write!(&mut encoder, "{text}")?;
        encoder.flush()?;
    }
    Ok(text_compress)
}

fn decompress_text(text_compressed: Vec<u8>) -> Result<String, io::Error> {
    let mut text = String::new();
    {
        let mut decoder = GzDecoder::new(&text_compressed[..]);
        decoder.read_to_string(&mut text)?;
    }
    Ok(text)
}

impl PipelineStep for Compressor {
    type IN = DocumentHeading;
    type OUT = DocumentCompressed;
    type ARG = ();

    async fn transform(input: Self::IN, _: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        let document = format!("{input}");
        let document = compress_text(&document).map_err(CompressionError::Io)?;
        let compressed = Self::OUT {
            document,
            article_title: input.article_title,
            access_date: input.access_date,
            modification_date: input.access_date,
        };
        Ok(vec![compressed])
    }

    fn args(&self) -> Self::ARG {}

    fn name() -> String {
        String::from("Compressor")
    }
}
