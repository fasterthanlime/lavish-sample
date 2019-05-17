use lavish_rpc as rpc;
use rpc::Atom;
use serde::Serialize;
use std::io::Cursor;
use std::marker::{PhantomData, Unpin};

use bytes::*;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use futures_codec::{Decoder, Encoder, Framed};

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
    pub fn new(io: T) -> Self
    where
        T: AsyncRead + AsyncWrite + Sized,
    {
        let codec = Codec::<P, NP, R> {
            phantom: PhantomData,
            pr: PendingRequests {},
        };
        let framed = Framed::new(io, codec);
        let (sink, stream) = framed.split();
        Self { sink, stream }
    }
}

pub struct Codec<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    phantom: PhantomData<(P, NP, R)>,
    pr: PendingRequests,
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

        println!("res = {:#?}", res);

        match res {
            Ok(m) => {
                // TODO: set pending
                let len = src.len();
                src.split_to(pos as usize);
                println!("[decoder] decoded messages from {}/{} bytes", pos, len);
                Ok(Some(m))
            }
            Err(DecErr::InvalidDataRead(_)) => need_more(),
            Err(DecErr::InvalidMarkerRead(_)) => need_more(),
            Err(DecErr::Syntax(_)) => need_more(),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

struct PendingRequests {}

impl rpc::PendingRequests for PendingRequests {
    fn get_pending<'a>(&self, _id: u32) -> Option<&'a str> {
        None
    }
}
