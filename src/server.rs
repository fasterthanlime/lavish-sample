use futures::executor;
use futures::prelude::*;

use romio::tcp::TcpListener;

use super::services::sample;

#[allow(clippy::needless_lifetimes)]
pub async fn run(
    mut listener: TcpListener,
    pool: &executor::ThreadPool,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut incoming = listener.incoming();

    if let Some(conn) = incoming.next().await {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] <- {}", addr);
        conn.set_nodelay(true)?;

        sample::peer_with_handler(conn, pool, (), |mut h| {
            use sample::get_cookies::*;
            register(&mut h, async move |call| {
                let mut cookies: Vec<sample::Cookie> = Vec::new();
                cookies.push(sample::Cookie {
                    key: "ads".into(),
                    value: "no".into(),
                });
                cookies.push(sample::Cookie {
                    key: "user".into(),
                    value: "John Doe".into(),
                });

                cookies.push(sample::Cookie {
                    key: "user-agent".into(),
                    value: sample::get_user_agent::call(&call.handle, ())
                        .await?
                        .user_agent,
                });

                Ok(Results { cookies })
            });
        })?;
    }
    Ok(())
}
