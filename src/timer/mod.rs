mod delay;

pub use delay::Delay;

use chrono::Duration;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::executor::reactor;

pub struct Timer {
    key: reactor::Key,
    reactor: reactor::Handle,
}

impl Timer {
    pub fn start(reactor: reactor::Handle, duration: Duration) -> Result<Self, failure::Error> {
        Ok(Self {
            key: reactor.add_timer(duration, true),
            reactor,
        })
    }
}

impl futures::Stream for Timer {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match futures::ready!(self.reactor.poll_elapsed(cx, self.key)) {
            Ok(_) => Poll::Ready(Some(())),
            Err(e) => panic!("timer error: {}", e),
        }
    }
}
