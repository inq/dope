use std::net::{SocketAddr, ToSocketAddrs};
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, Stream};

use super::TcpStream;
use crate::executor::reactor;

pub struct TcpListener {
    inner: std::net::TcpListener,
    register: reactor::Register,
}

pub struct Incoming {
    inner: TcpListener,
}

impl Stream for Incoming {
    type Item = Result<TcpStream, failure::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (socket, _) = ready!(self.inner.poll_accept(cx))?;
        Poll::Ready(Some(Ok(socket)))
    }
}

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(
        reactor: reactor::Handle,
        addr: A,
    ) -> Result<Self, failure::Error> {
        let inner = std::net::TcpListener::bind(addr)?;
        inner.set_nonblocking(true)?;
        Ok(Self {
            inner,
            register: reactor::Register::new(reactor),
        })
    }

    pub fn incoming(self) -> Incoming {
        Incoming { inner: self }
    }

    fn poll_accept(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(TcpStream, SocketAddr), failure::Error>> {
        let (std_stream, addr) = match self.inner.accept() {
            Ok(pair) => pair,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let fd = self.inner.as_raw_fd();
                self.register.register(cx, fd)?;
                return Poll::Pending;
            }
            Err(e) => return Poll::Ready(Err(failure::Error::from(e))),
        };
        log::info!("accepted: {:?}", addr);

        let res = TcpStream::new(self.register.clone_reactor(), std_stream)?;
        Poll::Ready(Ok((res, addr)))
    }
}
