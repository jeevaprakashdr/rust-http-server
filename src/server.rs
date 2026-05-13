use std::{io::{self, BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}};

pub(crate) struct Server {
    host: String,
    port: u16,
}

impl Server {
    fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub(crate) fn init() -> Self {
        Server::new("127.0.0.1".to_string(), 4221)
    }

    pub(crate) fn start(&self) {
        let listener = self.listner().unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Accepted new connection");
                    let mut streamer = TcpStreamWrapper::new(&stream);
                    let content = streamer.read().unwrap();
                    if content.contains(&"GET") {
                        streamer.write("HTTP/1.1 200 OK\r\n\r\n".to_string()).unwrap()
                    }
                }
                Err(e) => eprintln!("error: {}", e)
            }
        }
    }

    fn listner(&self) -> io::Result<TcpListener> {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port));
        listener
            .inspect_err(|e| eprintln!("Failed to bind TCPListener. {}", e))
            .inspect(|l| println!("listening on {:?}", l))
    }
}

struct TcpStreamWrapper<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

impl<'a> TcpStreamWrapper<'a> {
    fn new(stream: &'a TcpStream) -> Self {
        Self {
            reader: BufReader::new(&stream),
            writer: BufWriter::new(&stream),
        }
    }

    fn read(&mut self) -> io::Result<String> {
        let mut buffer = [0; 512];

        let bytes_count = self
            .reader
            .read(&mut buffer[..])
            .inspect_err(|e| eprintln!("Failed to read from stream. {}", e))
            .unwrap();

        let str = str::from_utf8(&buffer[..bytes_count])
            .inspect(|x| println!("read content original: {x}"))
            .unwrap();

        Ok(str.to_string())
    }

    fn write(&mut self, content: String) -> io::Result<()> {
        self.writer.write_all(content.as_bytes())
    }
}
