#![feature(async_await)]
#![warn(clippy::all)]

use futures::executor;
use futures::prelude::*;

use romio::tcp::{TcpListener, TcpStream};

mod proto;

pub mod sleep;

use lavish_rpc::System;
use proto::protocol;

static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    let mut executor = executor::ThreadPool::new().unwrap();
    let pool = executor.clone();

    executor.run(async {
        let addr = ADDR.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("[server] <> {}", addr);

        let client = client(pool.clone()).map_err(|e| eprintln!("client error: {:#?}", e));
        let server =
            server(listener, pool.clone()).map_err(|e| eprintln!("server error: {:#?}", e));
        futures::future::join(client, server).await;
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

        let mut h = proto::Handler::new(futures::lock::Mutex::new(state));
        use proto::sample::{print, reverse, show_stats};
        print::register(&mut h, async move |call| {
            let s = reverse::call(&call.handle, reverse::Params { s: call.params.s })
                .await?
                .s;
            println!("[server] {}", s);

            {
                let mut state = call.state.lock().await;
                state.total_characters += s.len();
            }

            Ok(())
        });

        show_stats::register(&mut h, async move |call| {
            println!(
                "[server] Total characters printed: {}",
                call.state.lock().await.total_characters
            );
            Ok(())
        });

        System::new(protocol(), Some(h), conn, pool.clone())?;
    }
    Ok(())
}

async fn client(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;

    let mut h = proto::Handler::new(());

    {
        use proto::sample::{print, reverse, show_stats};
        reverse::register(&mut h, async move |call| {
            Ok(reverse::Results {
                s: call.params.s.chars().rev().collect(),
            })
        });

        let rpc_system = System::new(protocol(), Some(h), conn, pool.clone())?;
        let handle = rpc_system.handle();

        let mut reversed = true;
        for line in &sample_lines() {
            print::call(
                &handle,
                print::Params {
                    s: line.clone(),
                    reversed,
                },
            )
            .await?;
            reversed = !reversed;
        }

        show_stats::call(&handle, ()).await?;
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
