mod kevent;

use crate::executor::reactor;
use libc;
use std::os::unix::io::RawFd;

#[derive(Debug, Fail)]
pub(in crate::executor::reactor) enum Error {
    #[fail(display = "kqueue returned -1")]
    Kqueue,
    #[fail(display = "kevent returned -1")]
    Kevent,
}

pub(in crate::executor::reactor) struct Kqueue {
    kq: RawFd,
    events: Vec<libc::kevent>,
}

impl Kqueue {
    pub fn new() -> Result<Self, Error> {
        let res = unsafe { libc::kqueue() };
        if res == -1 {
            return Err(Error::Kqueue);
        }
        Ok(Self {
            kq: res,
            events: Vec::with_capacity(16),
        })
    }

    fn manage_event(
        &mut self,
        ident: usize,
        filter: i16,
        flags: u16,
        fflags: u32,
        data: isize,
        udata: usize,
    ) -> Result<(), failure::Error> {
        let changes = vec![libc::kevent {
            ident: ident as libc::uintptr_t,
            filter,
            flags,
            fflags,
            data,
            udata: udata as *mut _,
        }];
        let res = unsafe {
            libc::kevent(
                self.kq,
                changes.as_ptr(),
                changes.len() as i32,
                ::std::ptr::null_mut(),
                0,
                std::ptr::null(),
            )
        };
        if res == -1 {
            log::error!("kevent");
            Err(Error::Kevent.into())
        } else {
            Ok(())
        }
    }

    pub fn add_fd_read(&mut self, fd: usize, key: reactor::Key) -> Result<(), failure::Error> {
        self.manage_event(
            fd,
            libc::EVFILT_READ,
            libc::EV_ADD | libc::EV_ENABLE,
            0,
            0,
            key.inner(),
        )
    }

    pub fn add_fd_write(&mut self, fd: usize, key: reactor::Key) -> Result<(), failure::Error> {
        self.manage_event(
            fd,
            libc::EVFILT_WRITE,
            libc::EV_ADD | libc::EV_ENABLE,
            0,
            0,
            key.inner(),
        )
    }

    pub fn remove_fd(&mut self, fd: usize) -> Result<(), failure::Error> {
        self.manage_event(
            fd,
            libc::EVFILT_READ,
            libc::EV_DELETE | libc::EV_DISABLE,
            0,
            0,
            0,
        )
    }

    pub fn add_timer(
        &mut self,
        ident: usize,
        duration: chrono::Duration,
        key: reactor::Key,
        repeat: bool,
    ) -> Result<(), failure::Error> {
        self.manage_event(
            ident,
            libc::EVFILT_TIMER,
            libc::EV_ADD | libc::EV_ENABLE | if repeat { 0 } else { libc::EV_ONESHOT },
            0,
            duration.num_milliseconds() as isize,
            key.inner(),
        )
    }

    pub fn add_signal(&mut self, signal: i32, key: reactor::Key) -> Result<(), failure::Error> {
        self.manage_event(
            signal as usize,
            libc::EVFILT_SIGNAL,
            libc::EV_ADD | libc::EV_ENABLE,
            0,
            0,
            key.inner(),
        )
    }

    fn fetch_events(&mut self) -> Result<(), Error> {
        unsafe {
            let res = libc::kevent(
                self.kq,
                std::ptr::null(),
                0,
                self.events.as_mut_ptr(),
                self.events.capacity() as i32,
                std::ptr::null(),
            );
            if res == -1 {
                return Err(Error::Kevent);
            } else {
                self.events.set_len(res as usize);
            }
        }
        Ok(())
    }

    pub fn poll(&mut self) -> Vec<reactor::Key> {
        use num_traits::FromPrimitive;

        self.fetch_events().unwrap();
        self.events
            .iter()
            .map(|e| {
                let filter = e.filter;
                debug!(
                    "polling: {} ident: {} filter: {} {:?} data: {} udata: {} fflags: {}",
                    e.flags & libc::EV_EOF,
                    e.ident as i32,
                    filter,
                    kevent::Filter::from_i16(filter as i16),
                    e.data as i32,
                    e.udata as i32,
                    e.fflags as i32,
                );
                reactor::Key::from(e.udata as usize)
            })
            .collect()
    }
}
