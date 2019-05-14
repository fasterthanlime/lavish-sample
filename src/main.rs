mod badsock;
mod proto;
mod support;
use serde::Serialize;

use futures::Future;
use futures::{future, future::Either};
use std::time::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio_io::AsyncRead;

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
        let exec = rt.executor();

        let addr = addr.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("[server] bound");

        let server = listener
            .incoming()
            .map_err(|e| eprintln!("accept failed: {:?}", e))
            .for_each(move |sock| {
                println!("[server] accepted");

                sock.set_nodelay(true).unwrap();
                let (_reader, writer) = sock.split();

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
                    (writer, lines, first_item),
                    move |(mut writer, mut lines, item)| {
                        Delay::new(Instant::now() + Duration::from_millis(80)).then(move |_| {
                            match item {
                                Some(line) => {
                                    let mut buf: Vec<u8> = Vec::new();
                                    let m = proto::Message::request(
                                        0,
                                        proto::Params::double_Print(proto::double::print::Params {
                                            s: line.into(),
                                        }),
                                    );

                                    buf.resize(0, 0);
                                    let mut ser = rmp_serde::Serializer::new_named(&mut buf);
                                    m.serialize(&mut ser).unwrap();

                                    Either::A(
                                        badsock::write_two_halves(writer, buf)
                                            .map_err(|e| println!("i/o error: {:#?}", e))
                                            .and_then(move |(writer, _)| {
                                                let next_item = lines.pop();
                                                Ok(future::Loop::Continue((
                                                    writer, lines, next_item,
                                                )))
                                            }),
                                    )
                                }
                                None => {
                                    println!("shutting down writer");
                                    writer.shutdown().unwrap();
                                    Either::B(future::result(Ok(future::Loop::Break(()))))
                                }
                            }
                        })
                    },
                );
                exec.spawn(myloop);

                Ok(())
            })
            .map_err(|e| eprintln!("server error {:?}", e));

        rt.block_on(server).unwrap();
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
