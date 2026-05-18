use std::io::Write;

use flate2::{Compression, write::GzEncoder};

pub(crate) fn gzip(sub_path: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(sub_path).unwrap();
    encoder.finish().unwrap()
}
