mod node;

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{RawWaker, RawWakerVTable, Waker};

use node::{Node, Task};

#[derive(Default)]
pub struct Scheduler {
    inner: Rc<RefCell<Inner>>,
}

pub struct Handle {
    inner: Weak<RefCell<Inner>>,
}

#[derive(Default)]
struct Inner {
    nodes: VecDeque<Node>,
}

impl Scheduler {
    pub fn handle(&self) -> Handle {
        Handle {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub fn waker(&self) -> Waker {
        let ptr = Rc::into_raw(self.inner.clone()) as *const ();
        let vtable =
            &RawWakerVTable::new(waker::clone, waker::wake, waker::wake_by_ref, waker::drop);
        unsafe { Waker::from_raw(RawWaker::new(ptr, vtable)) }
    }

    pub fn schedule(&mut self, item: Pin<Task>) {
        log::info!("schedule");
        let node = Node {
            scheduler: self.handle(),
            item: Cell::new(Some(item)),
        };

        self.inner.borrow_mut().nodes.push_back(node);
        log::info!("schedule: {}", self.inner.borrow().nodes.len());
    }

    pub fn tick(&self) {
        use std::task::{Context, Poll};
        log::debug!("tick: {} nodes", self.inner.borrow().nodes.len());
        let popped = self.inner.borrow_mut().nodes.pop_front();
        if let Some(node) = popped {
            let mut task = node.item.take().unwrap();
            let node = Rc::new(node);
            let ptr = &node as &Node as *const Node as *const ();
            let vtable = &RawWakerVTable::new(
                node::waker::clone,
                node::waker::wake,
                node::waker::wake_by_ref,
                node::waker::drop,
            );
            let waker = unsafe { Waker::from_raw(RawWaker::new(ptr, vtable)) };
            let mut cx = Context::from_waker(&waker);
            match task.as_mut().poll(&mut cx) {
                Poll::Ready(_) => {
                    log::warn!("Ready");
                }
                Poll::Pending => {
                    log::warn!("Pending: {}", Rc::strong_count(&node));
                    node.item.replace(Some(task));
                }
            }
        }
    }
}

mod waker {
    use super::Inner;
    use std::rc::Rc;
    use std::task::{RawWaker, RawWakerVTable};

    pub(super) unsafe fn clone(data: *const ()) -> RawWaker {
        let rc: Rc<Inner> = Rc::from_raw(data as *const super::Inner);
        let cloned = rc.clone();
        log::warn!(
            "waker::clone ({}, {})",
            Rc::strong_count(&rc),
            Rc::weak_count(&rc)
        );
        std::mem::forget(rc);
        std::mem::forget(cloned);

        let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        RawWaker::new(data, vtable)
    }

    pub(super) unsafe fn wake(data: *const ()) {
        log::warn!("waker::wake");
        let _rc: Rc<Inner> = Rc::from_raw(data as *const Inner);
    }

    pub(super) unsafe fn wake_by_ref(data: *const ()) {
        log::warn!("waker::wake_by_ref");
        let rc: Rc<Inner> = Rc::from_raw(data as *const Inner);
        std::mem::forget(rc);
    }

    pub(super) unsafe fn drop(data: *const ()) {
        log::warn!("waker::drop");
        std::mem::drop(Rc::<Inner>::from_raw(data as *const Inner));
    }
}
