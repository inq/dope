use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::executor::scheduler;

pub(super) type Task = Box<dyn Future<Output = Result<(), failure::Error>>>;

pub(super) struct Node {
    pub(super) scheduler: scheduler::Handle,
    pub(super) item: Cell<Option<Pin<Task>>>,
}

impl Node {
    fn wake(this: Rc<Node>, forget: bool) {
        if forget {
            let cloned = this.clone();
            std::mem::forget(cloned);
        }
        if let Some(inner) = this.scheduler.inner.upgrade() {
            inner
                .borrow_mut()
                .nodes
                .push_back(Rc::try_unwrap(this).ok().unwrap());
        } else {
            unreachable!();
        }
    }
}

pub(super) mod waker {
    use super::Node;
    use crate::executor::scheduler;
    use std::rc::Rc;
    use std::task::{RawWaker, RawWakerVTable};

    pub(in crate::executor::scheduler) unsafe fn clone(data: *const ()) -> RawWaker {
        log::debug!("node::waker::clone");
        let rc: Rc<scheduler::Inner> = Rc::from_raw(data as *const scheduler::Inner);
        let cloned = rc.clone();
        std::mem::forget(rc);
        std::mem::forget(cloned);

        let vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        RawWaker::new(data, vtable)
    }

    pub(in crate::executor::scheduler) unsafe fn wake(data: *const ()) {
        log::debug!("node::waker::wake");
        let node: Rc<Node> = Rc::from_raw(data as *const Node);
        Node::wake(node, false);
    }

    pub(in crate::executor::scheduler) unsafe fn wake_by_ref(data: *const ()) {
        log::debug!("node::waker::wake_by_ref");
        let node: Rc<Node> = Rc::from_raw(data as *const Node);
        Node::wake(node, true);
    }

    pub(in crate::executor::scheduler) unsafe fn drop(_: *const ()) {}
}
