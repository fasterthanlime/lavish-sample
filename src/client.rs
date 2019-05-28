use futures::executor;

use romio::tcp::TcpStream;

use super::services::sample;

#[allow(clippy::needless_lifetimes)]
pub async fn run(pool: &executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = super::ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;
    struct ClientState {
        user_agent: String,
        asked_for_user_agent: bool,
    };
    let state = futures::lock::Mutex::new(ClientState {
        user_agent: "lavish-sample/0.2.0".into(),
        asked_for_user_agent: false,
    });
    let client = sample::peer_with_handler(conn, pool, state, |mut h| {
        sample::get_user_agent::register(&mut h, async move |call| {
            let mut state = call.state.lock().await;
            state.asked_for_user_agent = true;
            Ok(sample::get_user_agent::Results {
                user_agent: state.user_agent.clone(),
            })
        });
    })?;

    let cookies = sample::get_cookies::call(&client, ()).await?.cookies;
    println!("Cookies = {:#?}", cookies);

    // println!(
    //     "Asked for ua? = {:#?}",
    //     state.lock().await.asked_for_user_agent
    // );

    Ok(())
}
