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

struct OptionOf<T>(pub Option<T>)
where
    T: LavishObject;

impl<'a, T> LavishObject for OptionOf<T>
where
    T: LavishObject,
{
    fn serialize<W: Write>(&self, wr: &mut W) -> Result<(), ValueWriteError> {
        match &self.0 {
            Some(v) => v.serialize(wr)?,
            None => wr
                .write_all(&[rmp::Marker::Null.to_u8()])
                .map_err(ValueWriteError::InvalidMarkerWrite)?,
        };

        Ok(())
    }
}

struct StringOf<'a>(pub &'a str);

impl<'a> LavishObject for StringOf<'a> {
    fn serialize<W: Write>(&self, wr: &mut W) -> Result<(), ValueWriteError> {
        use rmp::encode::*;
        write_str(wr, self.0)?;

        Ok(())
    }
}


struct ArrayOf<'a, T>(pub &'a [T])
where
    T: LavishObject;

impl<'a, T> LavishObject for ArrayOf<'a, T>
where
    T: LavishObject,
{
    fn serialize<W: Write>(&self, wr: &mut W) -> Result<(), ValueWriteError> {
        use rmp::encode::*;
        write_array_len(wr, self.0.len() as u32)?;
        for item in self.0 {
            item.serialize(wr)?;
        }

        Ok(())
    }
}

impl LavishObject for services::sample::Cookie {
    fn serialize<W: Write>(&self, wr: &mut W) -> Result<(), ValueWriteError> {
        use rmp::encode::*;
        write_array_len(wr, 3)?;
        StringOf(&self.key).serialize(wr)?;
        StringOf(&self.value).serialize(wr)?;
        OptionOf(self.comment.as_ref().map(|x| StringOf(&x))).serialize(wr)?;

        Ok(())
    }
}

fn serialize_sample() -> Result<(), Box<dyn Error + 'static>> {
    fn print_payload(slice: &[u8]) {
        println!("================================ payload ================================");
        println!("{:?}", slice.hex_dump());
        println!("=========================================================================");
    }

    let cookies = get_cookies();

    {
        let mut buf = Buf::new();
        ArrayOf(&cookies[..]).serialize(&mut buf)?;
        print_payload(&buf[..]);
    }

    {
        let mut buf = Buf::new();
        let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
        serde::Serialize::serialize(&cookies, &mut ser)?;
        print_payload(&buf[..]);
    }

    benchmarks::run();

    Ok(())
}

fn get_cookies() -> Vec<services::sample::Cookie> {
    return vec![
        services::sample::Cookie {
            key: "title".into(),
            value: "Knytt Underground".into(),
            comment: Some("Open for collabs".into()),
        },
        services::sample::Cookie {
            key: "title".into(),
            value: "Super Mario Maker".into(),
            comment: None,
        },
        services::sample::Cookie {
            key: "title".into(),
            value: "Overland".into(),
            comment: None,
        },
        services::sample::Cookie {
            key: "title".into(),
            value: "XCOM 2".into(),
            comment: None,
        },
        services::sample::Cookie {
            key: "title".into(),
            value: "Civilization V".into(),
            comment: None,
        },
    ];
}

use netbuf::Buf;

mod benchmarks {
    use bencher::*;

    use super::*;

    fn compact_serialize(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            ArrayOf(&cookies[..]).serialize(&mut buf).unwrap();
        });
    }

    fn serde_serialize(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap();
        });
    }

    benchmark_group!(serialize, compact_serialize, serde_serialize);
    benchmark_main!(serialize);

    pub fn run() {
        main();
    }
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
