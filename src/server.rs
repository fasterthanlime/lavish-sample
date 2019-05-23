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

        System::new(proto::protocol(), Some(h), conn, pool.clone())?;
    }
    Ok(())
}
