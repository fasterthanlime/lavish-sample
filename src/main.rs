#![feature(async_await)]

use futures::executor;
use futures::prelude::*;
use std::pin::Pin;

use romio::tcp::{TcpListener, TcpStream};

use lavish_rpc as rpc;

mod proto;
pub mod sleep;
mod support;

use support::{Handler, Protocol, RpcSystem};

use sleep::*;

static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    let mut executor = executor::ThreadPool::new().unwrap();
    let pool = executor.clone();

    executor.run(async {
        futures::future::join(client(pool.clone()), server(pool.clone())).await;
    });
}

fn protocol() -> Protocol<proto::Params, proto::NotificationParams, proto::Results> {
    Protocol::new()
}

struct ServerHandler {}

impl Handler<proto::Params, proto::Results> for ServerHandler {
    fn handle(
        &self,
        params: proto::Params,
    ) -> Pin<Box<dyn Future<Output = Result<proto::Results, String>> + Send + '_>> {
        Box::pin(async move {
            sleep_ms(120).await;
            match params {
                proto::Params::double_Double(params) => Ok(proto::Results::double_Double(
                    proto::double::double::Results { x: params.x * 2 },
                )),
                proto::Params::double_Print(params) => {
                    println!("[server] client says: {}", params.s);
                    Ok(proto::Results::double_Print(
                        proto::double::print::Results {},
                    ))
                }
            }
        })
    }
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

        let handler = Box::new(ServerHandler {});
        RpcSystem::new(protocol(), Some(handler), conn, pool.clone())?;
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

    let mut rpc_system = RpcSystem::new(protocol(), None, conn, pool.clone())?;

    for line in &sample_lines() {
        let res = rpc_system
            .call(proto::Params::double_Print(proto::double::print::Params {
                s: line.clone(),
            }))
            .await?;
        println!("[server] res = {:#?}", res);
        sleep_ms(300).await;
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