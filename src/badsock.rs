use futures::future::Future;
use tokio::io::{write_all, AsyncWrite};
use tokio_io::io::Window;

pub fn write_two_halves<A, T>(a: A, b: T) -> impl Future<Error = std::io::Error>
where
    A: AsyncWrite,
    T: AsRef<[u8]>,
{
    use std::time::*;
    use tokio::timer::Delay;
    let len = b.as_ref().len();
    let mid = len / 2;
    let mut b = Window::new(b);

    b.set_end(mid);

    write_all(a, b).and_then(move |(a, mut window)| {
        println!("wrote first part, sleeping");
        let when = Instant::now() + Duration::from_millis(1000);
        let delay = Delay::new(when);
        delay
            .map_err(|e| println!(">>>>>>>>>>\ndelay error: {:#?}\n>>>>>>>>>>", e))
            .then(move |delay| {
                println!("well, our delay's here: {:#?}", delay);
                println!("writing second part");
                window.set_end(len);
                window.set_start(mid);
                write_all(a, window)
            })
    })
}
