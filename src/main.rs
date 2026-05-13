#[allow(unused_imports)]
use std::net::TcpListener;
use std::{io::{Read, Write}, net::TcpStream};

fn main() {
    println!("Logs http server will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let content = read_stream(&mut stream);
                if content.contains(&"GET") {
                    let response = "HTTP/1.1 200 OK\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn read_stream(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];

    let bytes_count = stream
        .read(&mut buffer[..])
        .inspect_err(|e| eprintln!("Failed to read from stream. {}", e))
        .unwrap();

    let content = str::from_utf8(&buffer[..bytes_count]).unwrap();
    println!("streamed content {:?}", content);
    content.to_string()
}
