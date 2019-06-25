#![warn(clippy::all)]

pub mod services;

mod client;
mod server;

use std::error::Error;

fn main() {
    color_backtrace::install();
    env_logger::init();

    serialize_sample().unwrap();
    network_sample().unwrap();
    benchmarks::run();
}

use lavish::facts::{self, Factual};
use pretty_hex::PrettyHex;
use services::sample::protocol::ProtocolMapping;

fn get_protocol_mapping() -> ProtocolMapping {
    ProtocolMapping::default()
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
        let tt = get_protocol_mapping();
        cookies.write(&tt, &mut buf)?;
        print_payload(&buf[..]);
    }

    Ok(())
}

fn get_cookies() -> Vec<services::sample::Cookie> {
    return vec![
        services::sample::Cookie {
            key: "name".into(),
            value: "Knytt Underground".into(),
            comment: Some("Open for collabs".into()),
        },
        services::sample::Cookie {
            key: "name".into(),
            value: "Super Mario Maker".into(),
            comment: None,
        },
        services::sample::Cookie {
            key: "name".into(),
            value: "Overland".into(),
            comment: None,
        },
        services::sample::Cookie {
            key: "name".into(),
            value: "XCOM 2".into(),
            comment: None,
        },
        services::sample::Cookie {
            key: "name".into(),
            value: "Civilization V".into(),
            comment: None,
        },
    ];
}

lazy_static::lazy_static! {
    static ref EMOJIS: Vec<services::sample::Emoji> = {
        let payload = std::fs::read_to_string("emojis.json").unwrap();
        let emojis = json::parse(&payload).unwrap();

        match emojis {
            json::JsonValue::Object(obj) => {
                let mut res = Vec::<services::sample::Emoji>::new();
                for entry in obj.iter() {
                    res.push(services::sample::Emoji {
                        shortcode: entry.0.to_string(),
                        image_url: entry.1.to_string(),
                    });
                }
                res
            }
            val => panic!("emojis.json should bean object, but is {:#?}", val),
        }
    };
}

use netbuf::Buf;

mod benchmarks {
    use bencher::*;

    use super::*;

    fn ser_facts(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let tt = get_protocol_mapping();
        bench.iter(|| {
            buf.consume(buf.len());
            EMOJIS.write(&tt, &mut buf).unwrap();
        });
    }

    fn deser_facts(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let tt = get_protocol_mapping();
        EMOJIS.write(&tt, &mut buf).unwrap();

        bench.iter(|| {
            let mut slice = &buf[..];
            let mut r = facts::Reader::new(&mut slice);
            Vec::<services::sample::Emoji>::read(&mut r).unwrap();
        });
    }

    benchmark_group!(ser, ser_facts);
    benchmark_group!(deser, deser_facts);
    benchmark_main!(ser, deser);

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
