use flate2::{read::GzDecoder, write::GzEncoder};
use std::io::{self, Read, Write};

pub(crate) fn compress_text(text: &str) -> Result<Vec<u8>, io::Error> {
    let mut text_compress = vec![];
    {
        let mut encoder = GzEncoder::new(&mut text_compress, flate2::Compression::new(9));
        write!(&mut encoder, "{text}")?;
        encoder.flush()?;
    }
    Ok(text_compress)
}

pub(crate) fn decompress_text(text_compressed: &Vec<u8>) -> Result<String, io::Error> {
    let mut text = String::new();
    {
        let mut decoder = GzDecoder::new(&text_compressed[..]);
        decoder.read_to_string(&mut text)?;
    }
    Ok(text)
}
