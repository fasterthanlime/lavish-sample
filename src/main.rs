#![feature(async_await)]

use futures::executor;
use futures::prelude::*;

use romio::tcp::{TcpListener, TcpStream};

use lavish_rpc as rpc;

mod proto;
pub mod sleep;
mod support;

use std::sync::{Arc, Mutex};

use sleep::*;

type RpcSystem<T> = support::RpcSystem<proto::Params, proto::NotificationParams, proto::Results, T>;

static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    let mut executor = executor::ThreadPool::new().unwrap();
    let pool = executor.clone();

    executor.run(async {
        futures::future::join(client(pool.clone()), server(pool.clone())).await;
    });
}

async fn server(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = ADDR.parse()?;
    let mut listener = TcpListener::bind(&addr)?;
    let mut incoming = listener.incoming();
    println!("[server] <> {}", addr);

    if let Some(conn) = incoming.next().await {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] <- {}", addr);

        conn.set_nodelay(true)?;

        let mut rpc_system = RpcSystem::new(conn, pool.clone())?;

        for line in &sample_lines() {
            sleep_ms(300).await;
            // let m = proto::Message::request(
            //     0,
            //     proto::Params::double_Print(proto::double::print::Params { s: line.clone() }),
            // );
            // sink.send(m).await?;
            rpc_system
                .call(proto::Params::double_Print(proto::double::print::Params {
                    s: line.clone(),
                }))
                .await?;
        }
    }

    println!("[server] XX");
    Ok(())
}

async fn client(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    sleep_ms(100).await;

    let addr = ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;

    let mut rpc_system = RpcSystem::new(conn, pool.clone())?;

    while let Some(m) = rpc_system.stream.next().await {
        match m? {
            rpc::Message::Request { params, id } => match params {
                proto::Params::double_Print(params) => {
                    rpc_system
                        .call(proto::Params::double_Print(proto::double::print::Params {
                            s: "got it!".into(),
                        }))
                        .await?;
                    println!("[client] !! ({}) {:?}", id, params.s);
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

    println!("[client] XX");
    Ok(())
}

fn sample_lines() -> Vec<String> {
    let text = "This is the first sentence. The second sentence is slighter longer. The third sentence is the longest of the three sentences.";
    text.split(".")
        .filter_map(|x| {
            let x = x.trim();
            if x == "" {
                None
            } else {
                Some(x.into())
            }
        })
        .collect()
}