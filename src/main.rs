#![feature(async_await)]

use futures::executor;
use futures::prelude::*;
use std::pin::Pin;

use romio::tcp::{TcpListener, TcpStream};

mod proto;
pub mod sleep;
mod support;

use lavish_rpc::Atom;

use support::{Handler, Protocol, RpcHandle, RpcSystem};

static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    let mut executor = executor::ThreadPool::new().unwrap();
    let pool = executor.clone();

    executor.run(async {
        let addr = ADDR.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("[server] <> {}", addr);
        futures::future::join(client(pool.clone()), server(listener, pool.clone())).await;
        println!("both futures completed");
    });
}

fn protocol() -> Protocol<proto::Params, proto::NotificationParams, proto::Results> {
    Protocol::new()
}

struct ServerHandler {}

impl Handler<proto::Params, proto::NotificationParams, proto::Results> for ServerHandler {
    fn handle(
        &self,
        mut h: RpcHandle<proto::Params, proto::NotificationParams, proto::Results>,
        params: proto::Params,
    ) -> Pin<Box<dyn Future<Output = Result<proto::Results, String>> + Send + '_>> {
        Box::pin(async move {
            match params {
                proto::Params::double_Double(params) => Ok(proto::Results::double_Double(
                    proto::double::double::Results { x: params.x * 2 },
                )),
                proto::Params::double_Print(params) => {
                    println!("[server] client says: {}", params.s);
                    sleep::sleep_ms(250).await;
                    match h
                        .call(proto::Params::double_Print(proto::double::print::Params {
                            s: params.s.chars().rev().collect(),
                        }))
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => eprintln!("[server] client errored: {:#?}", e),
                    };

                    Ok(proto::Results::double_Print(
                        proto::double::print::Results {},
                    ))
                }
            }
        })
    }
}

async fn server(
    mut listener: TcpListener,
    pool: executor::ThreadPool,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut incoming = listener.incoming();

    if let Some(conn) = incoming.next().await {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] <- {}", addr);

        conn.set_nodelay(true)?;

        RpcSystem::new(
            protocol(),
            Some(Box::new(ServerHandler {})),
            conn,
            pool.clone(),
        )?;
    }
    Ok(())
}

struct ClientHandler {}

impl Handler<proto::Params, proto::NotificationParams, proto::Results> for ClientHandler {
    fn handle(
        &self,
        mut _h: RpcHandle<proto::Params, proto::NotificationParams, proto::Results>,
        params: proto::Params,
    ) -> Pin<Box<dyn Future<Output = Result<proto::Results, String>> + Send + '_>> {
        Box::pin(async move {
            match params {
                proto::Params::double_Print(params) => {
                    println!("[client] server says: {}", params.s);
                    sleep::sleep_ms(250).await;
                    Ok(proto::Results::double_Print(
                        proto::double::print::Results {},
                    ))
                }
                _ => Err(format!("method unimplemented {}", params.method())),
            }
        })
    }
}

async fn client(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;

    let rpc_system = RpcSystem::new(
        protocol(),
        Some(Box::new(ClientHandler {})),
        conn,
        pool.clone(),
    )?;
    let mut handle = rpc_system.handle();

    for line in &sample_lines() {
        handle
            .call(proto::Params::double_Print(proto::double::print::Params {
                s: line.clone(),
            }))
            .await?;
    }

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