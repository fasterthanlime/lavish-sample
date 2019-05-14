use futures::future::Future;
use tokio::io::{write_all, AsyncWrite};
use tokio_io::io::Window;

pub fn write_two_halves<A, T>(a: A, b: T) -> impl Future<Item = (A, T), Error = std::io::Error>
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

    write_all(a, b)
        .and_then(move |(a, mut window)| {
            let when = Instant::now() + Duration::from_millis(20);
            Delay::new(when).then(move |_| {
                window.set_end(len);
                window.set_start(mid);
                write_all(a, window)
            })
        })
        .map(|(a, b)| (a, b.into_inner()))
}
