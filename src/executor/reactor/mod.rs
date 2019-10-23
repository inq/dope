mod dispatcher;
mod register;
mod sys;

use std::cell::RefCell;
use std::convert::TryInto;
use std::os::unix::io::RawFd;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll, Waker};
use sys::Kqueue;

use slab::Slab;

pub use dispatcher::{Dispatcher, Key};
pub use register::Register;

pub struct Reactor {
    inner: Rc<RefCell<Inner>>,
}

#[derive(Clone)]
pub struct Handle {
    inner: Weak<RefCell<Inner>>,
}

struct Inner {
    kqueue: Kqueue,
    dispatchers: Slab<Dispatcher>,
}

impl Inner {
    fn insert(&mut self, waker: Option<Waker>) -> Key {
        let has_waker = waker.is_some();
        let res = Key::from(self.dispatchers.insert(Dispatcher::new(waker)));
        log::info!("insert: {:?}, waker: {}", res, has_waker);
        res
    }

    fn set_context(&mut self, key: Key, cx: &Context<'_>) -> Result<(), failure::Error> {
        match self.dispatchers.get_mut(key.inner()) {
            Some(dispatcher) => {
                log::info!("set_waker: {:?}", key);
                dispatcher.set_waker(cx.waker().clone())?;
            }
            None => unreachable!(),
        }
        Ok(())
    }

    fn get_mut(&mut self, key: Key) -> Option<&mut Dispatcher> {
        self.dispatchers.get_mut(key.inner())
    }

    fn remove(&mut self, key: Key) {
        self.dispatchers.remove(key.inner());
    }
}

impl Reactor {
    pub fn new() -> Result<Self, failure::Error> {
        Ok(Self {
            inner: Rc::new(RefCell::new(Inner {
                kqueue: Kqueue::new()?,
                dispatchers: Slab::new(),
            })),
        })
    }

    pub fn poll(&mut self) -> Result<(), failure::Error> {
        log::info!("Reactor::poll (maybe block)");
        let polled = self.inner.borrow_mut().kqueue.poll();
        log::info!("polled: {:?}", polled);
        for key in polled {
            if let Some(dispatcher) = self.inner.borrow_mut().get_mut(key) {
                dispatcher.wake()?;
            } else {
                panic!()
            }
        }

        Ok(())
    }

    pub(super) fn handle(&self) -> Handle {
        Handle {
            inner: Rc::downgrade(&self.inner),
        }
    }
}

impl Handle {
    pub fn poll_elapsed(&self, cx: &Context<'_>, key: Key) -> Poll<Result<(), failure::Error>> {
        log::info!("poll_elapsed");
        if let Some(inner) = self.inner.upgrade() {
            if let Some(dispatcher) = inner.borrow_mut().get_mut(key) {
                if dispatcher.consume() {
                    return Poll::Ready(Ok(()));
                } else {
                    dispatcher.set_waker(cx.waker().clone())?;
                }
            } else {
                unreachable!()
            }
        }
        Poll::Pending
    }

    pub fn add_signal(&self, signal: i32) -> Key {
        if let Some(inner) = self.inner.upgrade() {
            let mut borrowed = inner.borrow_mut();
            let key = borrowed.insert(None);
            borrowed.kqueue.add_signal(signal, key).unwrap();
            key
        } else {
            unreachable!()
        }
    }

    pub fn add_timer(&self, duration: chrono::Duration, repeat: bool) -> Key {
        if let Some(inner) = self.inner.upgrade() {
            let ident_offset = 0x1000;

            let mut borrowed = inner.borrow_mut();
            let key = borrowed.insert(None);
            borrowed
                .kqueue
                .add_timer(key.inner() + ident_offset, duration, key, repeat)
                .unwrap();
            key
        } else {
            unreachable!()
        }
    }

    pub fn set_context(&self, key: Key, cx: &Context<'_>) -> Result<(), failure::Error> {
        if let Some(inner) = self.inner.upgrade() {
            inner.borrow_mut().set_context(key, cx)
        } else {
            unreachable!();
        }
    }

    pub fn register_fd(&self, cx: &Context<'_>, fd: RawFd) -> Result<Key, failure::Error> {
        log::warn!("register_fd: {:?}", fd);
        if let Some(inner) = self.inner.upgrade() {
            let mut borrowed = inner.borrow_mut();
            let key = borrowed.insert(Some(cx.waker().clone()));
            borrowed.kqueue.add_fd(fd.try_into()?, key)?;
            log::info!("ADDED: key {:?}, fd {}", key, fd);
            Ok(key)
        } else {
            unreachable!();
        }
    }

    pub fn unregister(&self, key: Option<Key>, fd: RawFd) -> Result<(), failure::Error> {
        if let Some(inner) = self.inner.upgrade() {
            let mut borrowed = inner.borrow_mut();
            if let Some(key) = key {
                borrowed.remove(key);
                borrowed.kqueue.remove_fd(fd.try_into()?)?;
            }
            Ok(())
        } else {
            unreachable!();
        }
    }
}
