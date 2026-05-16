use std::{
    fs::File,
    io::{self, Read},
    net::TcpListener,
    str, thread,
};

use clap::Parser;

use crate::{
    http::{self, HttpHeader, HttpResponse, HttpStatusCode},
    tcp::TcpStreamWrapper,
};

#[derive(clap::Parser, Clone)]
struct ServerSettings {
    #[arg(short, long)]
    directory: Option<String>,
}

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
        let settings = ServerSettings::parse().clone();

        for stream in listener.incoming() {
            let settings = settings.clone();
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        println!("Accepted new connection");
                        let mut streamer = TcpStreamWrapper::new(&stream);
                        let request = streamer.read().unwrap();
                        let response = handle_request(&request, &settings);
                        streamer.write(response).unwrap();
                    });
                }
                Err(e) => eprintln!("error: {}", e),
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

fn handle_request(request: &[u8], settings: &ServerSettings) -> String {
    match http::parse(request) {
        Some(request) => match request.get_path().as_slice() {
            [] => HttpResponse::with_status(HttpStatusCode::Ok).to_string(),
            [b"echo", sub_path] => HttpResponse::with_status(HttpStatusCode::Ok)
                .with_header(HttpHeader::ContentType.into(), "text/plain".to_string())
                .with_header(HttpHeader::ContentLength.into(), sub_path.len().to_string())
                .with_body(sub_path.to_vec())
                .to_string(),
            [b"user-agent"] => {
                let headers = request.get_headers();
                let user_agent = headers
                    .get::<&[u8]>(&HttpHeader::UserAgent.into())
                    .unwrap()
                    .trim_ascii();

                HttpResponse::with_status(HttpStatusCode::Ok)
                    .with_header(HttpHeader::ContentType.into(), "text/plain".to_string())
                    .with_header(
                        HttpHeader::ContentLength.into(),
                        user_agent.len().to_string(),
                    )
                    .with_body(user_agent.to_vec())
                    .to_string()
            }
            [b"files", file_name] => {
                let file_name = str::from_utf8(file_name).unwrap();
                let directory = settings.directory.clone().unwrap();
                let path = std::path::Path::new(&directory).join(file_name);

                match File::open(path) {
                    Ok(mut file) => {
                        let mut contents = Vec::<u8>::new();
                        let len = file.read_to_end(&mut contents).unwrap();
                        println!("{:?}", String::from_utf8(contents.clone()));
                        HttpResponse::with_status(HttpStatusCode::Ok)
                            .with_header(
                                HttpHeader::ContentType.into(),
                                "application/octet-stream".to_string(),
                            )
                            .with_header(HttpHeader::ContentLength.into(), len.to_string())
                            .with_body(contents)
                            .to_string()
                    }
                    Err(_) => {
                        HttpResponse::with_status(HttpStatusCode::NotFound).to_string()
                    },
                }
            }
            _ => HttpResponse::with_status(HttpStatusCode::NotFound).to_string(),
        },
        None => "unknown path".to_string(),
    }
}