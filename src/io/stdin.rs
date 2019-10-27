use futures::io::{AsyncRead, BufReader};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::executor::reactor;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "fcntl(F_GETFL) returned -1")]
    GetFl,
}

pub struct Stdin {
    inner: std::io::Stdin,
    register: reactor::Register,
}

pub fn stdin(reactor: reactor::Handle) -> Result<BufReader<Stdin>, failure::Error> {
    unsafe {
        let prev = libc::fcntl(libc::STDIN_FILENO, libc::F_GETFL);
        if prev < 0 {
            return Err(Error::GetFl.into());
        }
        libc::fcntl(libc::STDIN_FILENO, libc::F_SETFL, prev | libc::O_NONBLOCK);
    }
    Ok(BufReader::new(Stdin {
        inner: std::io::stdin(),
        register: reactor::Register::new(reactor),
    }))
}

impl AsyncRead for Stdin {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        log::debug!("poll_read");
        use std::io::Read;

        match self.inner.read(buf) {
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                log::warn!("WouldBlock");
                let fd = libc::STDIN_FILENO;
                self.register.register_read(cx, fd).unwrap();
                Poll::Pending
            }
            etc => Poll::Ready(etc),
        }
    }
}
