use lavish_rpc::Atom;
use std::marker::PhantomData;

use bytes::*;
use std::io::Cursor;
use tokio_io::{AsyncRead, AsyncWrite};

#[must_use = "futures do nothing unless polled"]
pub struct RpcSystem<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    _p: PhantomData<P>,
    _np: PhantomData<NP>,
    _r: PhantomData<R>,
    pr: PendingRequests,
}

impl<P, NP, R> RpcSystem<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    pub fn new<T>(io: T) -> tokio_codec::Framed<T, Self>
    where
        T: AsyncRead + AsyncWrite + Sized,
    {
        let system = Self {
            _p: PhantomData,
            _np: PhantomData,
            _r: PhantomData,
            pr: PendingRequests {},
        };
        tokio_codec::Decoder::framed(system, io)
    }
}

use tokio_codec::{Decoder, Encoder};

impl<P, NP, R> Encoder for RpcSystem<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    type Item = lavish_rpc::Message<P, NP, R>;
    type Error = std::io::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        println!("encode called");
        Err(std::io::ErrorKind::Other.into())
    }
}

impl<P, NP, R> Decoder for RpcSystem<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    type Item = lavish_rpc::Message<P, NP, R>;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        println!("decode called with {} bytes", src.len());
        if src.len() == 0 {
            return Ok(None);
        }

        let (pos, res) = {
            let cursor = Cursor::new(&src[..]);
            let mut deser = rmp_serde::Deserializer::from_read(cursor);
            let res = Self::Item::deserialize(&mut deser, &self.pr);
            (deser.position(), res)
        };

        match res {
            Ok(m) => {
                // TODO: set pending
                src.split_to(pos as usize);
                Ok(Some(m))
            }
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("rmp serde error: {:#?}", e),
            )),
        }
    }
}

struct PendingRequests {}

impl lavish_rpc::PendingRequests for PendingRequests {
    fn get_pending<'a>(&self, _id: u32) -> Option<&'a str> {
        None
    }
}
