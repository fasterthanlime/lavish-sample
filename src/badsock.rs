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

    futures::future::lazy(move || {
        b.set_end(mid);
        write_all(a, b)
    })
    .and_then(|(a, window)| {
        let when = Instant::now() + Duration::from_millis(250);
        Delay::new(when)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .map(|()| (a, window))
    })
    .and_then(move |(a, mut window)| {
        window.set_start(mid + 1);
        window.set_end(len - 1);
        write_all(a, window)
    })
}
