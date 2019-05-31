use super::services::sample;
use std::net::TcpListener;

pub fn run(mut listener: TcpListener) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut incoming = listener.incoming();

    if let Some(conn) = incoming.next() {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] <- {}", addr);
        conn.set_nodelay(true)?;

        sample::peer(conn).with_handler(|h| {
            h.on_get_cookies(move |call| {
                let mut cookies: Vec<sample::Cookie> = Vec::new();
                cookies.push(sample::Cookie {
                    key: "ads".into(),
                    value: "no".into(),
                });

                cookies.push(sample::Cookie {
                    key: "user-agent".into(),
                    value: call.client.get_user_agent()?.user_agent,
                });

                Ok(sample::get_cookies::Results { cookies })
            });

            h.on_reverse(move |call| {
                Ok(sample::reverse::Results {
                    s: call.params.s.chars().rev().collect(),
                })
            });

            h.on_ping(move |call| {
                // FIXME: this should be call.handle.ping
                call.client.ping__ping()?;

                Ok(())
            });
        })?;
    }
    Ok(())
}
