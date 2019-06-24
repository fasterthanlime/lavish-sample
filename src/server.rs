use super::services::sample;
use std::sync::Arc;

pub fn router() -> sample::server::Router<()> {
    let mut r = sample::server::Router::new(Arc::new(()));
    r.handle(sample::get_cookies, |call| {
        let mut cookies: Vec<sample::Cookie> = Vec::new();
        cookies.push(sample::Cookie {
            key: "ads".into(),
            value: "no".into(),
            comment: None,
        });

        cookies.push(sample::Cookie {
            key: "user-agent".into(),
            value: call
                .client
                .call(sample::get_user_agent::Params {})?
                .user_agent,
            comment: None,
        });

        Ok(sample::get_cookies::Results { cookies })
    });

    r.handle(sample::reverse, |call| {
        Ok(sample::reverse::Results {
            s: call.params.s.chars().rev().collect(),
        })
    });

    r.handle(sample::ping, move |call| {
        call.client.call(sample::ping::ping::Params {})?;

        if let Some(val) = std::env::var("SAMPLE_SHUTDOWN").ok() {
            if val == "1" {
                call.shutdown_runtime();
            }
        }

        Ok(sample::ping::Results {})
    });

    r.handle(sample::record_mood, move |call| {
        let mood = call.params.mood;
        println!("Recording mood {:#?} for day {:#?}", mood.mood, mood.day);

        Ok(sample::record_mood::Results {})
    });

    r
}
