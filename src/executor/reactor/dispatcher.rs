use std::task::Waker;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "no waker")]
    NoWaker,
}

pub struct Dispatcher {
    available: bool,
    waker: Option<Waker>,
}

impl Dispatcher {
    pub fn new(waker: Option<Waker>) -> Self {
        Self {
            available: false,
            waker,
        }
    }

    pub fn wake(&mut self) -> Result<(), failure::Error> {
        self.available = true;
        if let Some(waker) = self.waker.take() {
            waker.wake();
            Ok(())
        } else {
            Err(Error::NoWaker.into())
        }
    }

    pub fn consume(&mut self) -> bool {
        if self.available {
            self.available = false;
            true
        } else {
            false
        }
    }

    pub fn set_waker(&mut self, waker: Waker) -> Result<(), failure::Error> {
        // TODO: Confirm this
        if self.waker.replace(waker).is_some() {
            log::warn!("set_waker: overwriting");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Key(usize);

impl Key {
    pub fn inner(self) -> usize {
        self.0
    }
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
