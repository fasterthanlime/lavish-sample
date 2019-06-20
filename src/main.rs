#![warn(clippy::all)]

pub mod services;

mod client;
mod server;

use std::error::Error;

fn main() {
    color_backtrace::install();
    env_logger::init();

    benchmarks::run();
    // serialize_sample().unwrap();
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
        cookies.write(&tt, &mut buf)?;
        print_payload(&buf[..]);
    }

    {
        let mut buf = Buf::new();
        let mut ser = rmp_serde::encode::Serializer::new(&mut buf);
        serde::Serialize::serialize(&cookies, &mut ser)?;
        print_payload(&buf[..]);
    }

    {
        let mut buf = Buf::new();
        let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
        serde::Serialize::serialize(&cookies, &mut ser)?;
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

use netbuf::Buf;

mod benchmarks {
    use bencher::*;

    use super::*;

    fn ser_facts(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let tt = get_translation_tables();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            cookies.write(&tt, &mut buf).unwrap();
        });
    }

    fn deser_facts(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let tt = get_translation_tables();
        let cookies = get_cookies();
        cookies.write(&tt, &mut buf).unwrap();

        bench.iter(|| {
            let mut slice = &buf[..];
            let mut r = facts::Reader::new(&mut slice);
            Vec::<services::sample::Cookie>::read(&mut r).unwrap();
        });
    }

    fn ser_serde_index(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            let mut ser = rmp_serde::encode::Serializer::new(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap()
        });
    }

    fn deser_serde_index(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let tt = get_translation_tables();
        let cookies = get_cookies();
        {
            let mut ser = rmp_serde::encode::Serializer::new(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap();
        }

        bench.iter(|| {
            use serde::Deserialize;
            let mut deser = rmp_serde::decode::Deserializer::from_slice(&buf[..]);
            Vec::<services::sample::Cookie>::deserialize(&mut deser).unwrap();
        });
    }

    fn ser_serde_named(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        bench.iter(|| {
            buf.consume(buf.len());
            let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap();
        });
    }

    fn deser_serde_named(bench: &mut Bencher) {
        let mut buf = Buf::new();
        let cookies = get_cookies();
        {
            let mut ser = rmp_serde::encode::Serializer::new_named(&mut buf);
            serde::Serialize::serialize(&cookies, &mut ser).unwrap();
        }

        bench.iter(|| {
            use serde::Deserialize;
            let mut deser = rmp_serde::decode::Deserializer::from_slice(&buf[..]);
            Vec::<services::sample::Cookie>::deserialize(&mut deser).unwrap();
        });
    }

    benchmark_group!(ser, ser_facts, ser_serde_index, ser_serde_named);
    benchmark_group!(deser, deser_facts, deser_serde_index, deser_serde_named,);
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
