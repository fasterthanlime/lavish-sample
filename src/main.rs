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

mod facts;
use facts::Factual;
use pretty_hex::PrettyHex;

fn get_translation_tables() -> facts::TranslationTables {
    facts::TranslationTables {
        sample__Cookie: vec![Some(0), Some(1), Some(2)],
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
        let tt = get_translation_tables();
        facts::array_of(&cookies[..]).write(&tt, &mut buf)?;
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
        let tt = get_translation_tables();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            facts::array_of(&cookies[..]).write(&tt, &mut buf).unwrap();
        });
    }

    fn serde_compact_serialize(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            let mut ser = rmp_serde::encode::Serializer::new(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap();
        });
    }

    fn serde_named_serialize(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap();
        });
    }

    benchmark_group!(
        serialize,
        compact_serialize,
        serde_compact_serialize,
        serde_named_serialize
    );
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
