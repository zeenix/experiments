use std::{
    future::Future,
    io::{self, Result},
    os::{fd::AsFd, unix},
    pin::Pin,
    task::{Context, Poll, Waker},
    thread,
};

use rustix::event::{PollFd, PollFlags};

#[derive(Debug)]
pub struct UnixStream(unix::net::UnixStream);

impl super::UnixStream for UnixStream {
    fn pipe() -> Result<(Self, Self)> {
        unix::net::UnixStream::pair().and_then(|(s1, s2)| {
            s1.set_nonblocking(true)?;
            s2.set_nonblocking(true)?;

            Ok((Self(s1), Self(s2)))
        })
    }

    fn read<'r>(&'r mut self, buf: &'r mut [u8]) -> impl Future<Output = Result<usize>> + 'r {
        Read {
            stream: &mut self.0,
            buf,
        }
    }

    fn write<'r>(&'r mut self, buf: &'r [u8]) -> impl Future<Output = Result<usize>> + 'r {
        Write {
            stream: &mut self.0,
            buf,
        }
    }
}

#[derive(Debug)]
pub struct Read<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r mut [u8],
}

impl Future for Read<'_> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::Read;

        let this = self.get_mut();
        match this.stream.read(this.buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                wake_on_event(&this.stream, Event::Read, cx.waker());

                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[derive(Debug)]
pub struct Write<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r [u8],
}

impl Future for Write<'_> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::Write;

        let this = self.get_mut();
        match this.stream.write(this.buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                wake_on_event(&this.stream, Event::Write, cx.waker());

                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

fn wake_on_event<Fd: AsFd>(fd: &Fd, event: Event, waker: &Waker) {
    let fd = fd.as_fd().try_clone_to_owned().unwrap();
    let waker = waker.clone();

    thread::spawn(move || {
        poll(&fd, event);

        waker.wake();
    });
}

/// Poll the given FD for specified event.
fn poll<Fd: AsFd>(fd: &Fd, event: Event) {
    let flags = PollFlags::from(event);
    let poll_fd = PollFd::new(fd, flags);
    let mut poll_fds = [poll_fd];

    loop {
        let size = rustix::event::poll(&mut poll_fds, None).unwrap();
        if size == 1 {
            break;
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Event {
    Read,
    Write,
}

impl From<Event> for PollFlags {
    fn from(value: Event) -> PollFlags {
        match value {
            Event::Read => PollFlags::IN,
            Event::Write => PollFlags::OUT,
        }
    }
}
