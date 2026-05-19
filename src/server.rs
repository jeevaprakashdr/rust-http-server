use std::{
    fs::File,
    io::{self, Read, Write},
    net::TcpListener,
    str, thread,
};

use clap::Parser;

use crate::{
    encoder,
    http::{self, Encoding, HttpHeader, HttpResponse, HttpStatusCode},
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
                        loop {
                            let mut streamer = TcpStreamWrapper::new(&stream);
                            let request = streamer.read().unwrap();
                            if request.is_empty() {
                                break;
                            }

                            if let Some(http_request) = http::parse(request.as_slice()) {
                                let response = handle_request(http_request.clone(), &settings);
                                // println!("{:?}", String::from_utf8(response.clone()).unwrap());
                                streamer.write(response[0..].to_vec()).unwrap();

                                if http_request
                                    .get_headers()
                                    .contains_key(&String::from(HttpHeader::Connection).as_bytes())
                                {
                                    break;
                                }
                            }
                        }
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

fn handle_request(request: http::HttpRequest<'_>, settings: &ServerSettings) -> Vec<u8> {
    match request.get_method() {
        http::HttpVerb::Get => process_get(settings, request),
        http::HttpVerb::Post => process_post(settings, request),
        http::HttpVerb::Unknown(_) => HttpResponse::with_status(HttpStatusCode::NotFound).create(),
    }
}

fn process_post(settings: &ServerSettings, request: http::HttpRequest<'_>) -> Vec<u8> {
    match request.get_path().as_slice() {
        [b"files", file_name] => {
            let file_name = str::from_utf8(file_name).unwrap();
            let directory = settings.directory.clone().unwrap();
            let path = std::path::Path::new(&directory).join(file_name);
            match File::create(path) {
                Ok(mut file) => {
                    let buf = request.get_data();
                    file.write_all(buf).unwrap();
                    HttpResponse::with_status(HttpStatusCode::Created).create()
                }
                Err(_) => HttpResponse::with_status(HttpStatusCode::NotFound).create(),
            }
        }
        _ => HttpResponse::with_status(HttpStatusCode::NotFound).create(),
    }
}

fn process_get(settings: &ServerSettings, request: http::HttpRequest<'_>) -> Vec<u8> {
    let connection_close = request
        .get_headers()
        .contains_key(&String::from(HttpHeader::Connection).as_bytes());
    println!("connection close {}", connection_close);

    match request.get_path().as_slice() {
        [] => {
            let mut response = HttpResponse::with_status(HttpStatusCode::Ok);
            if connection_close {
                response.with_header(String::from(HttpHeader::Connection), "close".to_string());
            }
            response.create()
        }
        [b"echo", sub_path] => {
            let mut response = HttpResponse::with_status(HttpStatusCode::Ok)
                .with_header(HttpHeader::ContentType.into(), "text/plain".to_string());

            let default = response
                .with_header(HttpHeader::ContentLength.into(), sub_path.len().to_string())
                .with_body(sub_path.to_vec());

            if let Some(encoding) = request.get_encoding() {
                if encoding == Encoding::Gzip {
                    let encoded = encoder::gzip(sub_path);
                    response
                        .with_header(HttpHeader::ContentLength.into(), encoded.len().to_string())
                        .with_header(
                            HttpHeader::ContentEncoding.into(),
                            Encoding::Gzip.to_string(),
                        )
                        .with_body(encoded)
                        .create()
                } else {
                    default.create()
                }
            } else {
                default.create()
            }
        }
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
                .create()
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
                        .create()
                }
                Err(_) => HttpResponse::with_status(HttpStatusCode::NotFound).create(),
            }
        }
        _ => HttpResponse::with_status(HttpStatusCode::NotFound).create(),
    }
}
