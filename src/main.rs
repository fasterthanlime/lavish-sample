mod proto;

use lavish_rpc::Atom;
use os_pipe::pipe;
use rmp_serde::*;
use serde::Serialize;
use std::io;
use std::marker::PhantomData;

struct Transport<P, NP, R> {
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
    fn receive(&mut self) -> lavish_rpc::Message<P, NP, R> {
        decode::from_read(self).unwrap()
    }

    fn send(&mut self, m: lavish_rpc::Message<P, NP, R>) {
        m.serialize(&mut Serializer::new_named(self)).unwrap()
    }
}

fn main() {
    let (reader1, writer1) = pipe().unwrap();
    let (reader2, writer2) = pipe().unwrap();

    let transport = std::thread::spawn(move || {
        let mut transport = Transport::<proto::Params, proto::NotificationParams, proto::Results> {
            r: Box::new(reader1),
            w: Box::new(writer2),
            _p: PhantomData,
            _np: PhantomData,
            _r: PhantomData,
        };

        let m = proto::Message::request(
            1,
            proto::Params::double_Double(proto::double::double::Params { x: 128 }),
        );
        transport.send(m);
    });

    let receiver = std::thread::spawn(move || {
        let mut transport = Transport::<proto::Params, proto::NotificationParams, proto::Results> {
            r: Box::new(reader2),
            w: Box::new(writer1),
            _p: PhantomData,
            _np: PhantomData,
            _r: PhantomData,
        };

        let m = transport.receive();
        println!("received: {:#?}", m);
    });

    transport.join().unwrap();
    receiver.join().unwrap();
}
