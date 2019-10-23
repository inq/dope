use futures::io::{AsyncRead, AsyncWrite};
use std::net;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::executor::reactor;

pub struct TcpStream {
    inner: net::TcpStream,
    register: reactor::Register,
}

impl TcpStream {
    pub fn new(
        reactor: reactor::Handle,
        std_stream: net::TcpStream,
    ) -> Result<Self, failure::Error> {
        std_stream.set_nonblocking(true)?;
        log::debug!("TcpStream::new(fd: {})", std_stream.as_raw_fd());
        Ok(Self {
            register: reactor::Register::new(reactor),
            inner: std_stream,
        })
    }
}

impl AsyncRead for TcpStream {
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
                let fd = self.inner.as_raw_fd();
                self.register.register(cx, fd).unwrap();
                Poll::Pending
            }
            etc => Poll::Ready(etc),
        }
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        use std::io::Write;
        Poll::Ready(self.inner.write(buf))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.inner.shutdown(std::net::Shutdown::Both)?;
        let fd = self.inner.as_raw_fd();
        self.register.unregister(fd).unwrap();
        Poll::Ready(Ok(()))
    }
}
