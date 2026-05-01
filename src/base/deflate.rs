use std::io::{self, Read};
use flate2::read::DeflateDecoder;

/// Inflates raw DEFLATE data.
/// 1C uses raw DEFLATE without zlib or gzip headers.
pub fn inflate(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decoded_data = Vec::new();
    decoder.read_to_end(&mut decoded_data)?;
    Ok(decoded_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_inflate() {
        let original_data = b"Hello, 1C DEFLATE world!";
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original_data).unwrap();
        let compressed_data = encoder.finish().unwrap();

        let decompressed_data = inflate(&compressed_data).unwrap();
        assert_eq!(original_data, decompressed_data.as_slice());
    }
}
