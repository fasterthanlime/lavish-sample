#![warn(clippy::all)]

pub mod services;

mod client;
mod server;

fn main() {
    env_logger::init();

    // binds synchronously, serves in the background
    // `serve_once` only accepts one connection, then quits
    let server = lavish::serve_once(server::handler(), "localhost:0").unwrap();

    // do a few test calls;
    client::run(server.local_addr()).unwrap();

    // this makes sure the server shuts down when the client disconnects
    server.join().unwrap();
}
