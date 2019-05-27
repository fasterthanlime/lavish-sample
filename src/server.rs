use futures::executor;
use futures::prelude::*;

use romio::tcp::TcpListener;

use super::services::sample;
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

        let mut h = sample::Handler::new(());
        {
            use sample::get_cookies::*;
            register(&mut h, async move |_call| {
                let mut cookies: Vec<sample::Cookie> = Vec::new();
                cookies.push(sample::Cookie {
                    key: "ads".into(),
                    value: "no".into(),
                });
                cookies.push(sample::Cookie {
                    key: "user".into(),
                    value: "John Doe".into(),
                });

                Ok(Results { cookies })
            });
        }

        System::new(sample::protocol(), h, conn, pool.clone())?;
    }
    Ok(())
}
