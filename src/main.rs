mod badsock;
mod proto;
mod support;

use futures::Future;
use futures::{future, future::Either};
use std::time::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::Runtime;

type RpcSystem = support::RpcSystem<proto::Params, proto::NotificationParams, proto::Results>;

fn main() {
    let addr = "127.0.0.1:9596";

    let client_thread = std::thread::spawn(move || {
        let mut rt = Runtime::new().unwrap();

        println!("[client] waiting a bit...");
        std::thread::sleep(Duration::from_millis(200));

        let addr = addr.parse().unwrap();
        let sock = TcpStream::connect(&addr).wait().unwrap();
        sock.set_nodelay(true).unwrap();

        println!("[client] connected");
        let rpc_system = RpcSystem::new(sock);

        rt.block_on(
            rpc_system
                .for_each(|m| {
                    println!(
                        "🦀 {}",
                        format!("{:#?}", m)
                            .split("\n")
                            .map(|x| x.trim())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    Ok(())
                })
                .map_err(|err| {
                    println!("rpc system error: {}", err);
                    std::process::exit(1);
                })
                .and_then(|_| {
                    println!("reached the end of the rpc system");
                    Ok(())
                }),
        )
        .unwrap();

        println!("[client] shutting down on idle...");
        rt.shutdown_on_idle().wait().unwrap();
        println!("[client] has shut down!");
    });

    let server_thread = std::thread::spawn(move || {
        let mut rt = Runtime::new().unwrap();

        let addr = addr.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("[server] bound");

        let sock = listener.incoming().wait().next().unwrap().unwrap();
        println!("[server] accepted");

        sock.set_nodelay(true).unwrap();
        let rpc_system = RpcSystem::new(sock);

        use std::time::*;
        use tokio::timer::Delay;

        let text = "Lorem ipsum. To most of us, it’s a passage of
                meaningless Latin that fills websites or brochure layouts
                with text while waiting on writers to fill it with real copy.
                This is bad news for publishers. But if one of those
                publishers decided to use it themselves, they'd be getting
                it. When in doubt, try to find a copy that's hard to find, a
                better copy is available, or an original copy of the text.
                You're looking to write something that's both long and clear
                and that has strong copy. Use the following guidelines to get
                started. If it has little to do with your own work, make
                sure it's not a book. This one is simple. Don't do work on an
                unrelated thing. Unless your main thing is writing this post,
                use your main thing as your main thing, and write the other
                parts of your work as secondary things";
        let mut lines: Vec<String> = text
            .split("\n")
            .map(|x| x.trim())
            .collect::<Vec<_>>()
            .join(" ")
            .split(".")
            .map(|x| x.trim().into())
            .collect();
        lines.reverse();
        let first_item = lines.pop();

        let myloop = futures::future::loop_fn(
            (rpc_system, lines, first_item),
            move |(rpc_system, mut lines, item)| {
                Delay::new(Instant::now() + Duration::from_millis(80)).then(move |_| match item {
                    Some(line) => {
                        let m = proto::Message::request(
                            0,
                            proto::Params::double_Print(proto::double::print::Params {
                                s: line.into(),
                            }),
                        );
                        Either::A(
                            rpc_system
                                .send(m)
                                .map_err(|e| println!("i/o error: {:#?}", e))
                                .and_then(move |rpc_system| {
                                    let next_item = lines.pop();
                                    Ok(future::Loop::Continue((rpc_system, lines, next_item)))
                                }),
                        )
                    }
                    None => {
                        println!("shutting down rpc system");
                        rpc_system
                            .into_inner()
                            .shutdown(std::net::Shutdown::Both)
                            .unwrap();
                        Either::B(future::result(Ok(future::Loop::Break(()))))
                    }
                })
            },
        );
        rt.spawn(myloop);

        println!("[server] shutting down on idle...");
        rt.shutdown_on_idle().wait().unwrap();
        println!("[server] has shut down!");
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
