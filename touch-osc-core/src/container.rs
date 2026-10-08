//! The raw zlib container a `.tosc` file is wrapped in.
//!
//! See `docs/FORMAT.md`: a `.tosc` file is a zlib stream (RFC 1950, header
//! bytes `78 9c`) with no outer envelope. It decompresses directly to the
//! UTF-8 XML document.

use crate::error::{Error, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(Error::Decompress)?;
    Ok(out)
}

pub fn compress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_arbitrary_bytes() {
        let data = b"hello world, this is some xml-ish text <lexml></lexml>".to_vec();
        let compressed = compress(&data).unwrap();
        assert_eq!(&compressed[0..2], &[0x78, 0x9c]);
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
