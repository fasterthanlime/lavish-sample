use lavish_rpc::Atom;
use rmp_serde::*;
use serde::Serialize;
use std::io;
use std::marker::PhantomData;

//----- Transport

pub struct Transport<P, NP, R> {
    r: Box<io::Read>,
    w: Box<io::Write>,

    _p: PhantomData<P>,
    _np: PhantomData<NP>,
    _r: PhantomData<R>,
}

impl<P, NP, R> io::Read for &mut Transport<P, NP, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.r.read(buf)
    }
}

impl<P, NP, R> io::Write for &mut Transport<P, NP, R> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl<P, NP, R> Transport<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    pub fn new(r: Box<io::Read>, w: Box<io::Write>) -> Self {
        Self {
            r,
            w,
            _p: PhantomData,
            _np: PhantomData,
            _r: PhantomData,
        }
    }

    pub fn receive(&mut self, pr: &lavish_rpc::PendingRequests) -> lavish_rpc::Message<P, NP, R> {
        lavish_rpc::Message::<P, NP, R>::deserialize(&mut Deserializer::from_read(self), pr)
            .unwrap()
    }

    pub fn send(&mut self, m: lavish_rpc::Message<P, NP, R>) {
        m.serialize(&mut Serializer::new_named(self)).unwrap()
    }
}

//----- Peer

use std::collections::HashMap;

struct PendingRequests {
    requests: HashMap<u32, PendingRequest>,
}

struct PendingRequest {
    id: u32,
    method: &'static str,
}

impl std::cmp::PartialEq for PendingRequest {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

impl std::cmp::Eq for PendingRequest {}

pub struct Peer<P, NP, R> {
    pub transport: Transport<P, NP, R>,
    id_seed: u32,
    pending: PendingRequests,
}

impl<P, NP, R> Peer<P, NP, R>
where
    P: Atom,
    NP: Atom,
    R: Atom,
{
    pub fn new(transport: Transport<P, NP, R>) -> Self {
        Self {
            transport,
            id_seed: 0,
            pending: PendingRequests {
                requests: HashMap::new(),
            },
        }
    }

    pub fn notify(&mut self, params: NP) {
        let msg = lavish_rpc::Message::Notification { params };
        self.transport.send(msg);
    }

    pub fn call(&mut self, params: P) -> (Option<String>, R) {
        let id = self.id_seed;
        self.id_seed += 1;
        let method = params.method();
        let msg = lavish_rpc::Message::Request { id, params };
        self.pending
            .requests
            .insert(id, PendingRequest { id, method });
        self.transport.send(msg);

        let res = self.transport.receive(&self.pending);
        match res {
            lavish_rpc::Message::Response { id, error, results } => {
                if let Some(pending) = self.pending.requests.get(&id) {
                    return (error, results);
                } else {
                    unimplemented!()
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn receive(&mut self) -> lavish_rpc::Message<P, NP, R> {
        self.transport.receive(&self.pending)
    }
}

impl lavish_rpc::PendingRequests for PendingRequests {
    fn get_pending<'b>(&self, id: u32) -> Option<&'b str> {
        self.requests.get(&id).map(|x| x.method)
    }
}
