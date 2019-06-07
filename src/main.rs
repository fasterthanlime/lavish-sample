#![warn(clippy::all)]

pub mod services;

mod client;
mod server;

fn main() {
    env_logger::init();
    let addr = "127.0.0.1:9596";

    let server_handle = lavish::serve_once(server::handler(), addr).unwrap();
    client::run(addr).unwrap();
    // this makes sure the server shuts down when the client disconnects
    server_handle.join().unwrap();
}
