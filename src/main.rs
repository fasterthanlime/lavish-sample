mod badsock;
mod proto;
mod support;
use serde::Serialize;

use futures::Future;
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
                    println!("client read a message: {:#?}", m);
                    Ok(())
                })
                .map_err(|err| {
                    println!("rpc system error: {}", err);
                    std::process::exit(1);
                }),
        )
        .unwrap();
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
                let (_reader, mut writer) = sock.split();

                exec.spawn(futures::future::lazy(move || {
                    for i in 0..3 {
                        std::thread::sleep(Duration::from_secs(1));

                        for j in 0..3 {
                            println!("[client] writing request {}-{}", i, j);

                            let mut buf: Vec<u8> = Vec::new();
                            let m = proto::Message::request(
                                0,
                                proto::Params::double_Double(proto::double::double::Params {
                                    x: 128,
                                }),
                            );

                            buf.resize(0, 0);
                            let mut ser = rmp_serde::Serializer::new_named(&mut buf);
                            m.serialize(&mut ser).unwrap();

                            badsock::write_two_halves(&mut writer, buf).wait().unwrap();
                        }
                    }

                    Ok(())
                }));

                println!("[server] spawned task");
                // Ok(())
                futures::future::empty()
            })
            .map_err(|e| eprintln!("server error {:?}", e));

        rt.block_on(server).unwrap();
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
