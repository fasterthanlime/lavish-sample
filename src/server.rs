use super::services::sample;
use std::sync::Arc;

pub fn handler() -> sample::server::Handler<()> {
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

    h.on_ping(move |call| {
        // FIXME: this should be call.handle.ping
        // call.client.ping__ping()?;

        if let Some(val) = std::env::var("SAMPLE_SHUTDOWN").ok() {
            if val == "1" {
                call.shutdown_runtime();
            }
        }

        Ok(sample::ping::Results {})
    });
    h
}
