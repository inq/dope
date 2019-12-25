use std::task::Waker;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "no waker")]
    NoWaker,
}

#[derive(PartialEq)]
pub enum WakerOption {
    NeedWaker,
    None,
}

pub struct Dispatcher {
    available: bool,
    waker: Option<Waker>,
    waker_option: WakerOption,
}

impl Dispatcher {
    pub fn new(waker: Option<Waker>, waker_option: WakerOption) -> Self {
        Self {
            available: false,
            waker,
            waker_option,
        }
    }

    pub fn wake(&mut self) -> Result<(), failure::Error> {
        self.available = true;
        if let Some(waker) = self.waker.take() {
            waker.wake();
            Ok(())
        } else if self.waker_option != WakerOption::NeedWaker {
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
