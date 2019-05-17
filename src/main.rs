#![feature(async_await)]


use async_timer::oneshot::*;
use futures::prelude::*;
use std::time::Duration;
// mod badsock;
// mod proto;
// mod support;

// type RpcSystem<T> = support::RpcSystem<proto::Params, proto::NotificationParams, proto::Results, T>;

// use Delay from the tokio::timer module to sleep the task:
async fn sleep_ms(n: u64) {
    Timer::new(Duration::from_millis(n)).await;
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() {
    println!("One");

    sleep_ms(1000).await;
    println!("Two");

    sleep_ms(1000).await;
    println!("Three");
}

// fn oldmain () {
//     let addr = "127.0.0.1:9596";

//     let client_thread = std::thread::spawn(move || {
//         let mut rt = Runtime::new().unwrap();
//         let exec = rt.executor();

//         println!("[client] waiting a bit...");
//         std::thread::sleep(Duration::from_millis(200));

//         let addr = addr.parse().unwrap();
//         let sock = TcpStream::connect(&addr).wait().unwrap();
//         sock.set_nodelay(true).unwrap();

//         println!("[client] connected");
//         let rpc_system = RpcSystem::new(sock);

//         rt.spawn(badsock::print_after_wait(500, "kalamazoo".into()));

//         let sink = rpc_system.sink;
//         let stream = rpc_system.stream;

//         {
//             let m = proto::Message::request(
//                 0,
//                 proto::Params::double_Print(proto::double::print::Params { s: "ack".into() }),
//             );
//             exec.spawn(
//                 sink.send(m)
//                     .map_err(|e| {
//                         eprintln!("error sending ack: {:#?}", e);
//                     })
//                     .map(|_| ()),
//             );
//         }

//         rt.block_on(
//             stream
//                 .for_each(move |m| {
//                     println!(
//                         "🦀 {}",
//                         format!("{:#?}", m)
//                             .split("\n")
//                             .map(|x| x.trim())
//                             .collect::<Vec<_>>()
//                             .join(" ")
//                     );

//                     Ok(())
//                 })
//                 .map_err(|err| {
//                     println!("rpc system error: {}", err);
//                     std::process::exit(1);
//                 })
//                 .and_then(|_| {
//                     println!("reached the end of the rpc system");
//                     Ok(())
//                 }),
//         )
//         .unwrap();

//         println!("[client] shutting down on idle...");
//         rt.shutdown_on_idle().wait().unwrap();
//         println!("[client] has shut down!");
//     });

//     let server_thread = std::thread::spawn(move || {
//         let mut rt = Runtime::new().unwrap();

//         let addr = addr.parse().unwrap();
//         let listener = TcpListener::bind(&addr).unwrap();
//         println!("[server] bound");

//         let sock = listener.incoming().wait().next().unwrap().unwrap();
//         println!("[server] accepted");

//         sock.set_nodelay(true).unwrap();
//         let rpc_system = RpcSystem::new(sock);

//         use std::time::*;
//         use tokio::timer::Delay;

//         let text = "Lorem ipsum. To most of us, it’s a passage of
//                 meaningless Latin that fills websites or brochure layouts
//                 with text while waiting on writers to fill it with real copy.
//                 This is bad news for publishers. But if one of those
//                 publishers decided to use it themselves, they'd be getting
//                 it. When in doubt, try to find a copy that's hard to find, a
//                 better copy is available, or an original copy of the text.
//                 You're looking to write something that's both long and clear
//                 and that has strong copy. Use the following guidelines to get
//                 started. If it has little to do with your own work, make
//                 sure it's not a book. This one is simple. Don't do work on an
//                 unrelated thing. Unless your main thing is writing this post,
//                 use your main thing as your main thing, and write the other
//                 parts of your work as secondary things";
//         let mut lines: Vec<String> = text
//             .split("\n")
//             .map(|x| x.trim())
//             .collect::<Vec<_>>()
//             .join(" ")
//             .split(".")
//             .map(|x| x.trim().into())
//             .collect();
//         lines.reverse();
//         let first_item = lines.pop();

//         let sink = rpc_system.sink;
//         let stream = rpc_system.stream;

//         let myloop =
//             futures::future::loop_fn((sink, lines, first_item), move |(sink, mut lines, item)| {
//                 Delay::new(Instant::now() + Duration::from_millis(80)).then(move |_| match item {
//                     Some(line) => {
//                         let m = proto::Message::request(
//                             0,
//                             proto::Params::double_Print(proto::double::print::Params {
//                                 s: line.into(),
//                             }),
//                         );
//                         Either::A(
//                             sink.send(m)
//                                 .map_err(|e| println!("i/o error: {:#?}", e))
//                                 .and_then(move |sink| {
//                                     let next_item = lines.pop();
//                                     Ok(future::Loop::Continue((sink, lines, next_item)))
//                                 }),
//                         )
//                     }
//                     None => Either::B(future::result(Ok(future::Loop::Break(sink)))),
//                 })
//             });
//         rt.spawn(myloop.and_then(move |sink| {
//             println!("shutting down rpc system");
//             sink.reunite(stream)
//                 .unwrap()
//                 .into_inner()
//                 .shutdown(std::net::Shutdown::Both)
//                 .unwrap();
//             Ok(())
//         }));

//         println!("[server] shutting down on idle...");
//         rt.shutdown_on_idle().wait().unwrap();
//         println!("[server] has shut down!");
//     });

//     server_thread.join().unwrap();
//     client_thread.join().unwrap();
// }
