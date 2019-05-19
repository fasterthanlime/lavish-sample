#![feature(async_await)]

use futures::executor;
use futures::prelude::*;
use std::pin::Pin;

use romio::tcp::{TcpListener, TcpStream};

mod proto;
pub mod sleep;
mod support;

use lavish_rpc::Atom;

use support::{Handler, Protocol, RpcHandle, RpcSystem};

static ADDR: &'static str = "127.0.0.1:9596";

fn main() {
    let mut executor = executor::ThreadPool::new().unwrap();
    let pool = executor.clone();

    executor.run(async {
        let addr = ADDR.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("[server] <> {}", addr);
        futures::future::join(client(pool.clone()), server(listener, pool.clone())).await;
        println!("both futures completed");
    });
}

fn protocol() -> Protocol<proto::Params, proto::NotificationParams, proto::Results> {
    Protocol::new()
}

type HandlerRet = Pin<Box<dyn Future<Output = Result<proto::Results, String>> + Send + 'static>>;

struct PluggableHandler<'a> {
    double_print: Option<
        Box<
            (Fn(
                    RpcHandle<proto::Params, proto::NotificationParams, proto::Results>,
                    proto::double::print::Params,
                ) -> (Pin<
                    Box<
                        Future<Output = Result<proto::double::print::Results, String>>
                            + Send
                            + 'static,
                    >,
                >)) + Sync
                + Send
                + 'a,
        >,
    >,
}

impl<'a> PluggableHandler<'a> {
    fn on_double_print<F, FT>(&mut self, f: F)
    where
        F: Fn(
                RpcHandle<proto::Params, proto::NotificationParams, proto::Results>,
                proto::double::print::Params,
            ) -> FT
            + Sync
            + Send
            + 'a,
        FT: Future<Output = Result<proto::double::print::Results, String>> + Send + 'static,
    {
        self.double_print = Some(Box::new(move |h, params| Box::pin(f(h, params))))
    }
}

impl<'a> PluggableHandler<'a> {
    fn new() -> Self {
        Self { double_print: None }
    }
}

impl<'a> Handler<proto::Params, proto::NotificationParams, proto::Results, HandlerRet>
    for PluggableHandler<'a>
{
    fn handle(
        &self,
        h: RpcHandle<proto::Params, proto::NotificationParams, proto::Results>,
        params: proto::Params,
    ) -> HandlerRet {
        let method = params.method();
        match params {
            proto::Params::double_Print(params) => match self.double_print.as_ref() {
                Some(hm) => {
                    let res = hm(h, params);
                    Box::pin(async move { Ok(proto::Results::double_Print(res.await?)) })
                }
                None => Box::pin(async move { Err(format!("no handler for {}", method)) }),
            },
            _ => Box::pin(async move { Err(format!("no handler for {}", method)) }),
        }
    }
}

async fn server(
    mut listener: TcpListener,
    pool: executor::ThreadPool,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut incoming = listener.incoming();

    if let Some(conn) = incoming.next().await {
        let conn = conn?;
        let addr = conn.peer_addr()?;
        println!("[server] <- {}", addr);

        conn.set_nodelay(true)?;

        // let val = std::sync::Mutex::new(0u8);

        let mut ph = PluggableHandler::new();
        ph.on_double_print(async move |mut h, params| {
            println!("[server] client says: {}", params.s);
            match h
                .call(proto::Params::double_Print(proto::double::print::Params {
                    s: params.s.chars().rev().collect(),
                }))
                .await
            {
                Ok(_) => {}
                Err(e) => eprintln!("[server] client errored: {:#?}", e),
            };

            // {
            //     let mut val = val.lock().unwrap();
            //     *val += 1;
            //     println!("val = {}", *val);
            // }

            Ok(proto::double::print::Results {})
        });

        RpcSystem::new(protocol(), Some(ph), conn, pool.clone())?;
    }
    Ok(())
}

async fn client(pool: executor::ThreadPool) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let addr = ADDR.parse()?;
    let conn = TcpStream::connect(&addr).await?;
    let addr = conn.peer_addr()?;
    println!("[client] -> {}", addr);

    conn.set_nodelay(true)?;

    let mut ph = PluggableHandler::new();
    ph.on_double_print(async move |_h, params| {
        println!("[client] server says: {}", params.s);
        Ok(proto::double::print::Results {})
    });

    let rpc_system = RpcSystem::new(protocol(), Some(ph), conn, pool.clone())?;
    let mut handle = rpc_system.handle();

    for line in &sample_lines() {
        handle
            .call(proto::Params::double_Print(proto::double::print::Params {
                s: line.clone(),
            }))
            .await?;
    }

    Ok(())
}

fn sample_lines() -> Vec<String> {
    let text = "This is the first sentence. The second sentence is slighter longer. The third sentence is the longest of the three sentences.";
    text.split(".")
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