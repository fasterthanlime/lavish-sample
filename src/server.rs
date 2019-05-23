use futures::executor;
use futures::prelude::*;

use romio::tcp::TcpListener;

use super::proto;
use lavish_rpc::System;

pub async fn run(
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
            let s = {
                if call.params.reversed {
                    reverse::call(&call.handle, reverse::Params { s: call.params.s })
                        .await?
                        .s
                } else {
                    call.params.s
                }
            };
            println!("[server] {}", s);

            {
                let mut state = call.state.lock().await;
                state.total_characters += s.len();
            }

            Ok(())
        });

        {
            use proto::sample::get_cookies::*;
            use std::collections::HashMap;
            register(&mut h, async move |_call| {
                let mut cookies = HashMap::new();
                cookies.insert("ads".to_string(), None);
                cookies.insert("user_id".to_string(), Some("1235".to_string()));

                Ok(Results { cookies })
            })
        }

        {
            use proto::sample::reverse_list::*;
            register(&mut h, async move |call| {
                let mut output = call.params.input;
                output.reverse();
                Ok(Results { output })
            })
        }

        show_stats::register(&mut h, async move |call| {
            println!(
                "[server] Total characters printed: {}",
                call.state.lock().await.total_characters
            );
            Ok(())
        });

        System::new(proto::protocol(), Some(h), conn, pool.clone())?;
    }
    Ok(())
}
