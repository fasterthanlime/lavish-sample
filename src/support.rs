use lavish_rpc as rpc;
use rpc::Atom;
use serde::Serialize;
use std::io::Cursor;
use std::marker::{PhantomData, Unpin};
use std::pin::Pin;

use futures::lock::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use bytes::*;
use futures::channel::oneshot;
use futures::executor;
use futures::prelude::*;
use futures::stream::SplitSink;
use futures_codec::{Decoder, Encoder, Framed};

use futures::task::SpawnExt;

pub trait IO: AsyncRead + AsyncWrite + Send + Sized + Unpin + 'static {}
impl<T> IO for T where T: AsyncRead + AsyncWrite + Send + Sized + Unpin + 'static {}

#[derive(Clone, Copy)]
pub struct Protocol<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    phantom: PhantomData<(P, NP, R)>,
}

impl<P, NP, R> Protocol<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

pub trait Handler<P, NP, R, T>: Sync + Send
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    fn handle(
        &self,
        h: RpcHandle<P, NP, R, T>,
        params: P,
    ) -> Pin<Box<dyn Future<Output = Result<R, String>> + Send + '_>>;
}

pub struct RpcHandle<P, NP, R, T>
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    pr: Arc<Mutex<PendingRequests<P, NP, R>>>,
    sink: Arc<Mutex<SplitSink<Framed<T, Codec<P, NP, R>>, rpc::Message<P, NP, R>>>>,
}

impl<P, NP, R, T> RpcHandle<P, NP, R, T>
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    fn clone(&self) -> Self {
        Self {
            pr: self.pr.clone(),
            sink: self.sink.clone(),
        }
    }

    pub async fn call(
        &mut self,
        params: P,
    ) -> Result<rpc::Message<P, NP, R>, Box<dyn std::error::Error + 'static>> {
        let id = {
            println!("[rpc] ..locking pr for genid");
            let mut pr = self.pr.lock().await;
            pr.genid()
        };
        println!("[rpc] unlocking pr for genid");

        let method = params.method();
        let m = rpc::Message::Request { id, params };

        let (tx, rx) = oneshot::channel::<rpc::Message<P, NP, R>>();
        let req = PendingRequest { method, tx };

        {
            println!("[rpc] ..locking pr for insert");
            let mut pr = self.pr.lock().await;
            pr.reqs.insert(id, req);
            println!("[rpc] unlocking pr for insert");
        }

        {
            println!("[rpc] ..locking sink for insert");
            let mut sink = self.sink.lock().await;
            sink.send(m).await?;
            println!("[rpc] unlocking sink for insert");
        }

        Ok(rx.await.unwrap())
    }
}

pub struct RpcSystem<P, NP, R, T>
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    handle: RpcHandle<P, NP, R, T>,
}

impl<P, NP, R, T> RpcSystem<P, NP, R, T>
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    pub fn new(
        protocol: Protocol<P, NP, R>,
        handler: Option<Box<Handler<P, NP, R, T>>>,
        io: T,
        mut pool: executor::ThreadPool,
    ) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let pr = Arc::new(Mutex::new(PendingRequests::new(protocol)));

        let codec = Codec { pr: pr.clone() };
        let framed = Framed::new(io, codec);
        let (sink, mut stream) = framed.split();

        let handle = RpcHandle::<P, NP, R, T> {
            pr: pr.clone(),
            sink: Arc::new(Mutex::new(sink)),
        };
        let lh = handle.clone();
        let lpool = pool.clone();

        pool.spawn(async move {
            let href = Arc::new(handler);

            while let Some(m) = stream.next().await {
                let lh = lh.clone();
                let mut pool = lpool.clone();
                let href = href.clone();

                pool.spawn(async move {
                    match m {
                        Ok(m) => match m {
                            rpc::Message::Request { id, params } => {
                                println!("[rpc] received request");
                                let m = match href.as_ref() {
                                    Some(handler) => match handler.handle(lh.clone(), params).await
                                    {
                                        Ok(results) => rpc::Message::Response::<P, NP, R> {
                                            id,
                                            results: Some(results),
                                            error: None,
                                        },
                                        Err(error) => rpc::Message::Response::<P, NP, R> {
                                            id,
                                            results: None,
                                            error: Some(error),
                                        },
                                    },
                                    _ => rpc::Message::Response {
                                        id,
                                        results: None,
                                        error: Some(format!("no method handler")),
                                    },
                                };

                                {
                                    println!("[rpc] ..locking sink for handler response send");
                                    let mut sink = lh.sink.lock().await;
                                    sink.send(m).await.unwrap();
                                    println!("[rpc] unlocking sink for handler response send");
                                }
                            }
                            rpc::Message::Response { id, error, results } => {
                                println!("[rpc] received response");
                                let req = {
                                    let mut pr = lh.pr.lock().await;
                                    pr.reqs.remove(&id)
                                };
                                if let Some(req) = req {
                                    req.tx
                                        .send(rpc::Message::Response { id, error, results })
                                        .unwrap();
                                }
                            }
                            rpc::Message::Notification { .. } => unimplemented!(),
                        },
                        Err(e) => panic!(e),
                    }
                })
                .err()
                .map(|e| eprintln!("RPC error: {:#?}", e));
            }
        })
        .map_err(|_| "spawn error")?;

        Ok(Self { handle })
    }

    pub fn handle(&self) -> RpcHandle<P, NP, R, T> {
        self.handle.clone()
    }
}

pub struct Codec<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    pr: Arc<Mutex<PendingRequests<P, NP, R>>>,
}

impl<P, NP, R> Encoder for Codec<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    type Item = rpc::Message<P, NP, R>;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // TODO: check/improve resize logic
        let mut len = std::cmp::max(128, dst.capacity());
        dst.resize(len, 0);

        loop {
            let (cursor, res) = {
                let cursor = Cursor::new(&mut dst[..len]);
                let mut ser = rmp_serde::Serializer::new_named(cursor);
                let res = item.serialize(&mut ser);
                (ser.into_inner(), res)
            };
            use rmp_serde::encode::Error as EncErr;

            match res {
                Ok(_) => {
                    let pos = cursor.position();
                    dst.resize(pos as usize, 0);
                    return Ok(());
                }
                Err(EncErr::InvalidValueWrite(_)) => {
                    len *= 2;
                    dst.resize(len, 0);
                    continue;
                }
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
        }
    }
}

impl<P, NP, R> Decoder for Codec<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    type Item = rpc::Message<P, NP, R>;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }

        let (pos, res) = {
            let cursor = Cursor::new(&src[..]);
            let mut deser = rmp_serde::Deserializer::from_read(cursor);
            let res = {
                if let Some(pr) = self.pr.try_lock() {
                    Self::Item::deserialize(&mut deser, &*pr)
                } else {
                    // FIXME: futures_codec doesn't fit the bill
                    panic!("could not acquire lock in decode");
                }
            };
            (deser.position(), res)
        };

        use rmp_serde::decode::Error as DecErr;
        let need_more = || {
            println!("[decoder] need more than {} bytes", src.len());
            Ok(None)
        };

        match res {
            Ok(m) => {
                src.split_to(pos as usize);
                Ok(Some(m))
            }
            Err(DecErr::InvalidDataRead(_)) => need_more(),
            Err(DecErr::InvalidMarkerRead(_)) => need_more(),
            Err(DecErr::Syntax(_)) => need_more(),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

struct PendingRequest<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    method: &'static str,
    tx: oneshot::Sender<rpc::Message<P, NP, R>>,
}

struct PendingRequests<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    id: u32,
    reqs: HashMap<u32, PendingRequest<P, NP, R>>,
}

impl<P, NP, R> PendingRequests<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    fn new(_protocol: Protocol<P, NP, R>) -> Self {
        Self {
            id: 0,
            reqs: HashMap::new(),
        }
    }

    fn genid(&mut self) -> u32 {
        let res = self.id;
        self.id += 1;
        res
    }
}

impl<P, NP, R> rpc::PendingRequests for PendingRequests<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    fn get_pending(&self, id: u32) -> Option<&'static str> {
        self.reqs.get(&id).map(|req| req.method)
    }
}
