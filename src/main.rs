mod proto;
mod support;

use bytes::*;
use futures::Future;
use std::time::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio_io::AsyncRead;

type RpcSystem<T> = support::RpcSystem<proto::Params, proto::NotificationParams, proto::Results, T>;

fn main() {
    let addr = "127.0.0.1:9596";

    let client_thread = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        let exec = rt.executor();

        println!("[client] waiting a bit...");
        std::thread::sleep(Duration::from_millis(200));

        let addr = addr.parse().unwrap();
        let sock = TcpStream::connect(&addr).wait().unwrap();
        sock.set_nodelay(true).unwrap();

        println!("[client] connected");
        let (reader, writer) = sock.split();

        let rpc_system = RpcSystem::new(reader);
        exec.spawn(
            rpc_system
                .for_each(|()| {
                    println!("server stream did a turn");
                    Ok(())
                })
                .map_err(|err| {
                    println!("rpc system error: {}", err);
                    std::process::exit(1);
                }),
        );

        futures::lazy(|| {
            println!("[client] sending some bytes");
            tokio::io::write_all(
                writer,
                "This is a pretty long string, don't you think?".as_bytes(),
            )
        })
        .and_then(|(writer, result)| {
            println!("[client] sent some bytes {:?}", result);
            println!("[client] flushing");
            tokio::io::flush(writer)
        })
        .and_then(|_writer| {
            println!("[client] done flushing");
            Ok(())
        })
        .map_err(|e| eprintln!("[client] error: {:?}", e))
        .wait()
        .unwrap();

        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
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
                let (reader, writer) = sock.split();

                let mut buf = BytesMut::with_capacity(16);
                buf.resize(16, 0);
                let task = futures::future::loop_fn(
                    (reader, writer, buf),
                    move |(reader, writer, buf)| {
                        let when = Instant::now() + Duration::from_millis(250);
                        tokio::timer::Delay::new(when)
                            .map_err(|_e| {})
                            .and_then(move |()| {
                                tokio::io::read(reader, buf)
                                    .map_err(|e| eprintln!("[server] read error: {:?}", e))
                                    .map(|(reader, buf, n)| {
                                        let s = String::from_utf8_lossy(&buf[..n]);
                                        println!("[server] read {:?}", s);
                                        (reader, buf, n)
                                    })
                            })
                            .and_then(move |(reader, mut buf, n)| {
                                buf.truncate(n);
                                tokio::io::write_all(writer, buf)
                                    .map_err(|e| eprintln!("[server] write error: {:?}", e))
                                    .map(|(writer, buf)| {
                                        println!("[server] wrote!");
                                        futures::future::Loop::Continue((reader, writer, buf))
                                    })
                            })
                    },
                );
                exec.spawn(task);

                println!("[server] spawned task");
                Ok(())
            })
            .map_err(|e| eprintln!("server error {:?}", e));

        rt.spawn(server);
        rt.shutdown_on_idle().wait().unwrap();
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
