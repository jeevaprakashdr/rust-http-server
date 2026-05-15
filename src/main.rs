#[allow(unused_imports)]
use std::net::TcpListener;

use crate::server::Server;

mod server;
mod http;

fn main() {
    println!("Logs http server will appear here!");
    let server = Server::init();
    server.start();
}
