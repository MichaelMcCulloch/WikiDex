use flate2::{read::GzDecoder, write::GzEncoder};

use std::io::{self, Read, Write};

use crate::ingest::pipeline::document::{CompressedDocument, DocumentWithHeading};

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
    type IN = DocumentWithHeading;
    type OUT = CompressedDocument;
    type ARG = ();

    async fn transform(input: Self::IN, _: &Self::ARG) -> Vec<Self::OUT> {
        let DocumentWithHeading {
            document,
            heading,
            article_title,
            access_date,
            modification_date,
        } = input;
        let document = format!("{heading}\n\n{document}");
        let document = compress_text(&document).unwrap();
        let compressed = Self::OUT {
            document,
            article_title,
            access_date,
            modification_date,
        };
        vec![compressed]
    }

    fn args(&self) -> Self::ARG {}

    fn name() -> String {
        String::from("Compressor")
    }
}
