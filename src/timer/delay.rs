use chrono::Duration;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::executor::reactor;

pub struct Delay {
    key: reactor::Key,
    reactor: reactor::Handle,
}

impl Delay {
    pub fn start(reactor: reactor::Handle, duration: Duration) -> Result<Self, failure::Error> {
        Ok(Self {
            key: reactor.add_timer(duration, false),
            reactor,
        })
    }
}

impl Future for Delay {
    type Output = Result<(), failure::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match futures::ready!(self.reactor.poll_elapsed(cx, self.key)) {
            Ok(_) => Poll::Ready(Ok(())),
            Err(e) => panic!("timer error: {}", e),
        }
    }
}
