#![warn(clippy::all)]

pub mod services;

mod client;
mod server;

use std::error::Error;

fn main() {
    env_logger::init();

    serialize_sample().unwrap();
    // network_sample().unwrap();
}

use pretty_hex::PrettyHex;

use std::io::Write;

use rmp::encode::ValueWriteError;

trait LavishObject {
    fn serialize<W: Write>(&self, wr: &mut W) -> Result<(), ValueWriteError>;
}

impl LavishObject for services::sample::Cookie {
    fn serialize<W: Write>(&self, wr: &mut W) -> Result<(), ValueWriteError> {
        use rmp::encode::*;
        write_array_len(wr, 2)?;
        write_str(wr, &self.key)?;
        write_str(wr, &self.value)?;

        Ok(())
    }
}

fn serialize_sample() -> Result<(), Box<dyn Error + 'static>> {
    use netbuf::Buf;
    fn print_payload(slice: &[u8]) {
        println!("================================ payload ================================");
        println!("{:?}", slice.hex_dump());
        println!("=========================================================================");
    }

    let cookie = services::sample::Cookie {
        key: "model".into(),
        value: "Ford".into(),
    };

    {
        let mut buf = Buf::new();
        cookie.serialize(&mut buf)?;
        print_payload(&buf[..]);
    }

    {
        let mut buf = Buf::new();
        let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
        serde::Serialize::serialize(&cookie, &mut ser)?;
        print_payload(&buf[..]);
    }

    Ok(())
}

fn network_sample() -> Result<(), Box<dyn Error + 'static>> {
    // binds synchronously, serves in the background
    // `serve_once` only accepts one connection, then quits
    let server = lavish::serve_once(server::router(), "localhost:0")?;

    // do a few test calls;
    client::run(server.local_addr())?;

    // this makes sure the server shuts down when the client disconnects
    server.join().unwrap();

    Ok(())
}
