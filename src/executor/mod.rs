pub mod reactor;
pub mod scheduler;

use reactor::Reactor;
use scheduler::Scheduler;

use std::future::Future;
use std::pin::Pin;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Executor {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Clone)]
pub struct Handle {
    inner: Weak<RefCell<Inner>>,
}

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "dropped pointer")]
    DroppedPtr,
}

impl Handle {
    pub fn reactor(&self) -> Result<reactor::Handle, failure::Error> {
        Ok(self
            .inner
            .upgrade()
            .ok_or(Error::DroppedPtr)?
            .borrow()
            .reactor
            .handle())
    }

    pub fn spawn<F>(&self, future: F) -> Result<(), failure::Error>
    where
        F: Future<Output = Result<(), failure::Error>> + 'static,
    {
        self.inner
            .upgrade()
            .ok_or(Error::DroppedPtr)?
            .borrow_mut()
            .scheduler
            .schedule(Box::pin(future));
        Ok(())
    }
}

impl Executor {
    pub fn new() -> Result<Self, failure::Error> {
        Ok(Self {
            inner: Rc::new(RefCell::new(Inner {
                reactor: Reactor::new()?,
                scheduler: Scheduler::default(),
            })),
        })
    }

    pub fn handle(&self) -> Handle {
        Handle {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub fn block_on<F>(&self, mut future: F) -> Result<F::Output, failure::Error>
    where
        F: Future,
    {
        use std::task::{Context, Poll};

        let mut future = unsafe { Pin::new_unchecked(&mut future) };
        let waker = self.inner.borrow().scheduler.waker();
        let mut cx = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(e) => return Ok(e),
                Poll::Pending => {}
            }

            self.inner.borrow().scheduler.tick();
            self.inner.borrow_mut().reactor.poll()?;
        }
    }
}

struct Inner {
    reactor: Reactor,
    scheduler: Scheduler,
}
