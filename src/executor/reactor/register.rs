use std::os::unix::io::RawFd;
use std::task::Context;

use crate::executor::reactor;

pub struct Register {
    reactor: reactor::Handle,
    key: Option<reactor::Key>,
}

impl Register {
    pub fn new(reactor: reactor::Handle) -> Self {
        Self { reactor, key: None }
    }

    pub fn clone_reactor(&self) -> reactor::Handle {
        self.reactor.clone()
    }

    pub fn register_read(&mut self, cx: &mut Context<'_>, fd: RawFd) -> Result<(), failure::Error> {
        // TODO: Merge with write
        match self.key {
            Some(key) => self.reactor.set_context(key, cx),
            None => {
                let key = self.reactor.register_fd_read(cx, fd)?;
                self.key.replace(key);
                Ok(())
            }
        }
    }

    pub fn register_write(
        &mut self,
        cx: &mut Context<'_>,
        fd: RawFd,
    ) -> Result<(), failure::Error> {
        match self.key {
            Some(key) => self.reactor.set_context(key, cx),
            None => {
                let key = self.reactor.register_fd_write(cx, fd)?;
                self.key.replace(key);
                Ok(())
            }
        }
    }

    pub fn unregister(&self, fd: RawFd) -> Result<(), failure::Error> {
        self.reactor.unregister(self.key, fd)
    }
}
