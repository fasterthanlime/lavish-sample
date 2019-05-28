#![feature(async_await)]
#![warn(clippy::all)]

use futures::executor;
use futures::prelude::*;

use romio::tcp::TcpListener;

pub mod services;
pub mod sleep;

mod client;
mod server;

pub static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    env_logger::init();

    let mut executor = executor::ThreadPool::new().unwrap();
    let pool = executor.clone();

    executor.run(async {
        let addr = ADDR.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("[server] <> {}", addr);

        let client = client::run(&pool).map_err(|e| eprintln!("client error: {:#?}", e));
        let server = server::run(listener, &pool).map_err(|e| eprintln!("server error: {:#?}", e));
        futures::future::join(client, server).await;
    });
}
