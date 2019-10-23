use std::pin::Pin;
use std::task::{Context, Poll};

use crate::executor::reactor;

pub struct Signal {
    key: reactor::Key,
    reactor: reactor::Handle,
}

impl Signal {
    pub fn start(reactor: reactor::Handle, signal: i32) -> Result<Self, failure::Error> {
        Ok(Self {
            key: reactor.add_signal(signal),
            reactor,
        })
    }
}

impl futures::Stream for Signal {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match futures::ready!(self.reactor.poll_elapsed(cx, self.key)) {
            Ok(_) => Poll::Ready(Some(())),
            Err(e) => panic!("timer error: {}", e),
        }
    }
}
