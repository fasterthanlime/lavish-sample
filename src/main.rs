#![feature(async_await)]
#![warn(clippy::all)]

use futures::executor;
use futures::prelude::*;

use romio::tcp::{TcpListener, TcpStream};

mod proto;
mod support;

pub mod sleep;

use lavish_rpc::System;
use support::{protocol, PluggableHandler};

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

        struct ServerState {
            total_characters: usize,
        }

        let state = ServerState {
            total_characters: 0,
        };
        let mut ph = PluggableHandler::new(futures::lock::Mutex::new(state));

        ph.on_double_print(async move |mut call| {
            println!("[server] client says: {}", call.params.s);
            call.handle
                .call(proto::Params::double_Print(proto::double::print::Params {
                    s: call.params.s.chars().rev().collect(),
                }))
                .map_err(|e| format!("{:#?}", e))
                .await?;

            {
                let mut state = call.state.lock().await;
                state.total_characters += call.params.s.len();
                println!("[server] total characters = {}", state.total_characters);
            }

            Ok(proto::double::print::Results {})
        });

        System::new(protocol(), Some(ph), conn, pool.clone())?;
    }
    Ok(())
}

async fn client(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;

    let mut ph = PluggableHandler::new(());
    ph.on_double_print(async move |call| {
        println!("[client] server says: {}", call.params.s);
        Ok(proto::double::print::Results {})
    });

    let rpc_system = System::new(protocol(), Some(ph), conn, pool.clone())?;
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
    text.split('.')
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