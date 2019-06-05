use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use super::services::sample;

pub fn run() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let conn = TcpStream::connect(super::ADDR)?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;
    struct ClientState {
        user_agent: String,
        asked_for_user_agent: bool,
    }

    let state = Arc::new(Mutex::new(ClientState {
        user_agent: "lavish-sample/0.2.0".into(),
        asked_for_user_agent: false,
    }));

    let mut h = sample::client::Handler::new(state.clone());
    h.on_get_user_agent(move |call| {
        let mut state = call.state.lock()?;
        state.asked_for_user_agent = true;
        Ok(sample::get_user_agent::Results {
            user_agent: state.user_agent.clone(),
        })
    });
    type P = sample::protocol::Params;
    type NP = sample::protocol::NotificationParams;
    type R = sample::protocol::Results;
    let client = lavish::connect(lavish::Protocol::<P, NP, R>::new(), h, conn)?;
    let client = sample::client::Client::new(client);

    if let Ok(state) = state.lock() {
        println!("Asked for ua? = {:#?}", state.asked_for_user_agent);
    }

    let cookies = client.get_cookies(sample::get_cookies::Params {})?.cookies;
    println!("Cookies = {:?}", cookies);

    if let Ok(state) = state.lock() {
        println!("Asked for ua? = {:#?}", state.asked_for_user_agent);
    }

    let s = "rust";
    println!("s (original) = {}", s);
    let s = client.reverse(sample::reverse::Params { s: s.into() })?.s;
    println!("s (reversed) = {}", s);

    println!("Pinging server");

    // Wrong! We don't define `ping.ping`, so the server's call to us
    // is going to fail.
    client.ping(sample::ping::Params {})?;

    Ok(())
}
