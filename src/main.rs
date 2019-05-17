#![feature(async_await)]

use futures::executor;
use futures::prelude::*;

use async_timer::oneshot::*;
use romio::tcp::{TcpListener, TcpStream};
use std::time::Duration;

use lavish_rpc as rpc;

mod proto;
mod support;

type RpcSystem<T> = support::RpcSystem<proto::Params, proto::NotificationParams, proto::Results, T>;

static ADDR: &'static str = "127.0.0.1:9596";

async fn sleep_ms(n: u64) {
    if n > 0 {
        Timer::new(Duration::from_millis(n)).await;
    }
}

fn main() {
    let mut pool = executor::ThreadPool::new().unwrap();
    pool.run(async {
        futures::future::join(client(), server()).await;
    });
}

async fn server() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = ADDR.parse()?;
    let mut listener = TcpListener::bind(&addr)?;
    let mut incoming = listener.incoming();
    println!("[server] bound");

    if let Some(conn) = incoming.next().await {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] accepted connection from {}", addr);

        conn.set_nodelay(true)?;

        let rpc_system = RpcSystem::new(conn);
        let (mut sink, mut _stream) = (rpc_system.sink, rpc_system.stream);

        for line in &sample_lines() {
            sleep_ms(300).await;
            let m = proto::Message::request(
                0,
                proto::Params::double_Print(proto::double::print::Params { s: line.clone() }),
            );
            sink.send(m).await?;
        }
    }

    println!("[server] exiting");
    Ok(())
}

async fn client() -> Result<(), Box<dyn std::error::Error + 'static>> {
    sleep_ms(100).await;

    println!("[client] hello");

    let addr = ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] connected to {}", addr);

    conn.set_nodelay(true)?;

    let rpc_system = RpcSystem::new(conn);
    let (_sink, mut stream) = (rpc_system.sink, rpc_system.stream);

    while let Some(m) = stream.next().await {
        match m? {
            rpc::Message::Request { params, .. } => match params {
                proto::Params::double_Print(params) => {
                    println!("[client] {}", params.s);
                }
                _ => {
                    println!("[client] request: {:#?}", params);
                }
            },
            m => {
                println!("[client] message: {:#?}", m);
            }
        }
    }

    println!("[client] is done receiving message, shutting down");
    Ok(())
}

fn sample_lines() -> Vec<String> {
    let text = "Lorem ipsum. To most of us, it’s a passage of
                    meaningless Latin that fills websites or brochure layouts
                    with text while waiting on writers to fill it with real copy.
                    This is bad news for publishers. But if one of those
                    publishers decided to use it themselves, they'd be getting
                    it. When in doubt, try to find a copy that's hard to find, a
                    better copy is available, or an original copy of the text.";
    let mut lines: Vec<String> = text
        .split("\n")
        .map(|x| x.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .split(".")
        .map(|x| x.trim().into())
        .collect();
    lines.reverse();
    lines
}