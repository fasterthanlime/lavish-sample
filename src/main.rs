mod proto;

use os_pipe::pipe;
use rmp_serde::*;
use serde::Serialize;
use std::io;

struct Transport {
    r: Box<io::Read>,
    w: Box<io::Write>,
}

impl io::Read for &mut Transport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.r.read(buf)
    }
}

impl io::Write for &mut Transport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl Transport {
    fn receive(&mut self) -> proto::Message {
        decode::from_read(self).unwrap()
    }

    fn send(&mut self, m: proto::Message) {
        m.serialize(&mut Serializer::new_named(self)).unwrap()
    }
}

fn main() {
    let (reader1, writer1) = pipe().unwrap();
    let (reader2, writer2) = pipe().unwrap();

    let transport = std::thread::spawn(move || {
        let mut transport = Transport {
            r: Box::new(reader1),
            w: Box::new(writer2),
        };

        let m = proto::Message::request(
            1,
            proto::Params::double_Double(proto::double::double::Params { x: 128 }),
        );
        transport.send(m);
    });

    let receiver = std::thread::spawn(move || {
        let mut transport = Transport {
            r: Box::new(reader2),
            w: Box::new(writer1),
        };

        let m = transport.receive();
        println!("received: {:#?}", m);
    });

    transport.join().unwrap();
    receiver.join().unwrap();
}
