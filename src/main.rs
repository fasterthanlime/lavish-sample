#![warn(clippy::all)]

use std::net::TcpListener;

pub mod services;

mod client;
mod server;

pub static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    env_logger::init();

    let addr = ADDR.parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    println!("[server] <> {}", addr);

    let client = std::thread::spawn(move || {
        client::run().unwrap();
    });
    let server = std::thread::spawn(move || {
        server::run(listener).unwrap();
    });
    client.join();
    server.join();
}
