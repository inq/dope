use futures::io::{AsyncWrite, BufWriter};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::executor::reactor;

pub struct Stdout {
    inner: std::io::Stdout,
    register: reactor::Register,
}

pub fn stdout(reactor: reactor::Handle) -> Result<BufWriter<Stdout>, failure::Error> {
    log::info!("stdout!");
    Ok(BufWriter::new(Stdout {
        inner: std::io::stdout(),
        register: reactor::Register::new(reactor),
    }))
}

impl AsyncWrite for Stdout {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        use std::io::Write;

        match self.inner.write(buf) {
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                log::warn!("WouldBlock");
                let fd = libc::STDOUT_FILENO;
                self.register.register_write(cx, fd).unwrap();
                Poll::Pending
            }
            etc => Poll::Ready(etc),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), std::io::Error>> {
        unimplemented!()
    }
}
