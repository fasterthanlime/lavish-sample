use lavish_rpc as rpc;
use rpc::Atom;
use serde::Serialize;
use std::io::Cursor;
use std::marker::{PhantomData, Unpin};

use std::collections::HashMap;

use bytes::*;
use futures::channel::oneshot;
use futures::executor;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use futures_codec::{Decoder, Encoder, Framed};

use futures::task::SpawnExt;

use super::sleep::*;

pub trait IO: AsyncRead + AsyncWrite + Sized + Unpin {}
impl<T> IO for T where T: AsyncRead + AsyncWrite + Sized + Unpin {}

#[must_use = "futures do nothing unless polled"]
pub struct RpcSystem<P, NP, R, T>
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    pub id: u32,
    pub sink: SplitSink<Framed<T, Codec<P, NP, R>>, rpc::Message<P, NP, R>>,
    pub stream: SplitStream<Framed<T, Codec<P, NP, R>>>,
}

impl<P, NP, R, T> RpcSystem<P, NP, R, T>
where
    P: Atom,
    NP: Atom,
    R: Atom,
    T: IO,
{
    pub fn new(
        io: T,
        mut pool: executor::ThreadPool,
    ) -> Result<Self, Box<dyn std::error::Error + 'static>>
    where
        T: AsyncRead + AsyncWrite + Sized,
    {
        let codec = Codec::<P, NP, R> {
            phantom: PhantomData,
            pr: PendingRequests::new(),
        };
        let framed = Framed::new(io, codec);
        let (sink, stream) = framed.split();

        pool.spawn(async {
            let mut i = 0;
            loop {
                println!("Henlo {} from rpc system", i);
                sleep_ms(250).await;
                i += 1;
            }
        })
        .map_err(|_| "spawn error")?;

        Ok(Self {
            sink,
            stream,
            id: 0,
        })
    }

    pub async fn call(&mut self, params: P) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let id = self.id;
        self.id += 1;
        let m = rpc::Message::Request { id, params };
        self.sink.send(m).await?;
        Ok(())
    }
}

pub struct Codec<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    phantom: PhantomData<(P, NP, R)>,
    pr: PendingRequests<P, NP, R>,
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
            let res = Self::Item::deserialize(&mut deser, &self.pr);
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
    reqs: HashMap<u32, PendingRequest<P, NP, R>>,
}

impl<P, NP, R> PendingRequests<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    fn new() -> Self {
        Self {
            reqs: HashMap::new(),
        }
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
