use super::services::sample;
use std::sync::{Arc, Mutex};

pub fn run<A>(addr: A) -> Result<(), Box<dyn std::error::Error + 'static>>
where
    A: std::net::ToSocketAddrs,
{
    struct ClientState {
        user_agent: String,
        asked_for_user_agent: bool,
    }

    let state = Arc::new(Mutex::new(ClientState {
        user_agent: "lavish-sample/0.2.0".into(),
        asked_for_user_agent: false,
    }));

    let mut r = sample::client::Router::new(state.clone());
    r.handle(sample::get_user_agent, |call| {
        let mut state = call.state.lock()?;
        state.asked_for_user_agent = true;
        Ok(sample::get_user_agent::Results {
            user_agent: state.user_agent.clone(),
        })
    });

    r.handle(sample::ping::ping, |_call| {
        println!("Server just pinged us");
        Ok(sample::ping::ping::Results {})
    });

    let client = lavish::connect(r, addr)?.client();
    if let Ok(state) = state.lock() {
        println!("Asked for ua? = {:#?}", state.asked_for_user_agent);
    }

    let cookies = client.call(sample::get_cookies::Params {})?.cookies;
    println!("Cookies = {:?}", cookies);

    if let Ok(state) = state.lock() {
        println!("Asked for ua? = {:#?}", state.asked_for_user_agent);
    }

    let s = "rust";
    println!("s (original) = {}", s);
    let s = client.call(sample::reverse::Params { s: s.into() })?.s;
    println!("s (reversed) = {}", s);

    println!("Pinging server");
    client.call(sample::ping::Params {})?;

    client.call(sample::record_mood::Params {
        mood: sample::MoodRecord {
            mood: sample::Mood::Good,
            day: ::lavish::chrono::offset::Utc::now(),
        },
    })?;

    Ok(())
}
