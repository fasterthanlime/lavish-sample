use super::services::sample;
use std::net::TcpListener;
use std::sync::Arc;

pub fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut incoming = listener.incoming();

    if let Some(conn) = incoming.next() {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] <- {}", addr);
        conn.set_nodelay(true)?;

        let cloned_conn = conn.try_clone()?;

        // sample::peer(conn).with_handler(|h| {
        let mut h = sample::server::Handler::new(Arc::new(()));
        h.on_get_cookies(|call| {
            let mut cookies: Vec<sample::Cookie> = Vec::new();
            cookies.push(sample::Cookie {
                key: "ads".into(),
                value: "no".into(),
            });

            cookies.push(sample::Cookie {
                key: "user-agent".into(),
                value: call
                    .client
                    .get_user_agent(sample::get_user_agent::Params {})?
                    .user_agent,
            });

            Ok(sample::get_cookies::Results { cookies })
        });

        h.on_reverse(|call| {
            Ok(sample::reverse::Results {
                s: call.params.s.chars().rev().collect(),
            })
        });

        h.on_ping(move |_| {
            // FIXME: this should be call.handle.ping
            // call.client.ping__ping()?;

            if let Some(val) = std::env::var("SAMPLE_SHUTDOWN").ok() {
                if val == "1" {
                    cloned_conn.shutdown(std::net::Shutdown::Both)?;
                }
            }

            Ok(sample::ping::Results {})
        });

        let (mut runtime, _) = h.spawn(conn)?;
        runtime.join().unwrap();
    }
    Ok(())
}
