mod proto;
mod support;

use os_pipe::pipe;

type Transport = support::Transport<proto::Params, proto::NotificationParams, proto::Results>;
type Peer<'a> = support::Peer<proto::Params, proto::NotificationParams, proto::Results>;

fn main() {
    let (reader1, writer1) = pipe().unwrap();
    let (reader2, writer2) = pipe().unwrap();

    let client_thread = std::thread::spawn(move || {
        let transport = Transport::new(Box::new(reader1), Box::new(writer2));
        let mut peer = Peer::new(transport);

        let (_, r) = peer.call(proto::Params::double_Double(
            proto::double::double::Params { x: 128 },
        ));
        match r {
            proto::Results::double_Double(r) => {
                println!("result = {}", r.x);
            }
        }
    });

    let server_thread = std::thread::spawn(move || {
        let transport = Transport::new(Box::new(reader2), Box::new(writer1));
        let mut peer = Peer::new(transport);
        let m = peer.receive();
        println!("received: {:#?}", m);

        match m {
            lavish_rpc::Message::Request { id, params } => match params {
                proto::Params::double_Double(params) => {
                    let response = proto::Message::response(
                        id,
                        None,
                        proto::Results::double_Double(proto::double::double::Results {
                            x: params.x * 2,
                        }),
                    );
                    peer.transport.send(response);
                }
            },
            _ => unimplemented!(),
        }
    });

    client_thread.join().unwrap();
    server_thread.join().unwrap();
}
