use futures::executor;

use romio::tcp::TcpStream;

use super::services::sample;

pub async fn run(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = super::ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;
    let client = sample::client(conn, &pool)?;

    let cookies = sample::get_cookies::call(&client, ()).await?.cookies;
    println!("Cookies = {:#?}", cookies);

    Ok(())
}
