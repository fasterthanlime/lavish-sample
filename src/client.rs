use futures::executor;

use romio::tcp::TcpStream;

use super::proto;
use lavish_rpc::System;

pub async fn run(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = super::ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;

    let mut h = proto::Handler::new(());

    {
        use proto::sample::{print, reverse, show_stats};
        reverse::register(&mut h, async move |call| {
            Ok(reverse::Results {
                s: call.params.s.chars().rev().collect(),
            })
        });

        let rpc_system = System::new(proto::protocol(), Some(h), conn, pool.clone())?;
        let handle = rpc_system.handle();

        let mut reversed = true;
        for line in &sample_lines() {
            print::call(
                &handle,
                print::Params {
                    s: line.clone(),
                    reversed,
                },
            )
            .await?;
            reversed = !reversed;
        }

        show_stats::call(&handle, ()).await?;

        use proto::sample::get_cookies;
        let cookies = get_cookies::call(&handle, ()).await?.cookies;
        println!("[client] Our cookies are: {:#?}", cookies);

        let list = vec!["one", "two", "three"]
            .iter()
            .map(|&x| x.to_string())
            .collect();
        println!("[client] Initial list: {:?}", list);
        use proto::sample::reverse_list;
        let list = reverse_list::call(&handle, reverse_list::Params { input: list })
            .await?
            .output;
        println!("[client] Reversed list: {:?}", list);
    }

    Ok(())
}

fn sample_lines() -> Vec<String> {
    let text = "This is the first sentence. The second sentence is slighter longer. The third sentence is the longest of the three sentences.";
    text.split('.')
        .filter_map(|x| {
            let x = x.trim();
            if x == "" {
                None
            } else {
                Some(x.into())
            }
        })
        .collect()
}
