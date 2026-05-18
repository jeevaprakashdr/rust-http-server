use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

pub(crate) struct TcpStreamWrapper<'a>(BufReader<&'a TcpStream>, BufWriter<&'a TcpStream>);

impl<'a> TcpStreamWrapper<'a> {
    pub(crate) fn new(stream: &'a TcpStream) -> Self {
        Self(BufReader::new(stream), BufWriter::new(stream))
    }

    pub(crate) fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = [0; 512];

        let bytes_count = self
            .0
            .read(&mut buffer[..])
            .inspect_err(|e| eprintln!("Failed to read from stream. {}", e))
            .unwrap();

        println!(
            "read content original: {:?}",
            str::from_utf8(&buffer[..bytes_count]).unwrap()
        );

        Ok(buffer[..bytes_count].to_vec())
    }

    pub(crate) fn write(&mut self, content: Vec<u8>) -> io::Result<()> {
        self.1.write_all(content.as_slice())
    }
}
