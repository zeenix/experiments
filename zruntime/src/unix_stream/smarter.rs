//! Smarter async Unix stream — uses a reactor to wake the executor.
//!
//! This implementation fixes the [naive version](super::naive) by actually
//! **using** the [`Waker`] from the polling [`Context`]. When a non-blocking
//! read or write returns `WouldBlock`, instead of silently returning
//! `Pending`, it:
//!
//! 1. Clones the file descriptor and the waker.
//! 2. Spawns a background thread that calls [`poll(2)`] on the FD.
//! 3. When the FD becomes ready (data available for reading, or buffer
//!    space available for writing), the thread calls `Waker::wake()`.
//! 4. This unparks the [smarter executor's](crate::executor::smarter)
//!    thread, which re-polls the future and the I/O succeeds.
//!
//! This background-thread-per-event approach is intentionally simple (a
//! real runtime would use epoll/kqueue with a single reactor thread), but
//! it's enough to demonstrate the waker contract.
//!
//! [`poll(2)`]: rustix::event::poll

use std::{
    future::Future,
    io::{self, Result},
    os::{fd::AsFd, unix},
    pin::Pin,
    task::{Context, Poll, Waker},
    thread,
};

use rustix::event::{PollFd, PollFlags};

/// A non-blocking Unix domain stream socket (smarter version).
#[derive(Debug)]
pub struct UnixStream(unix::net::UnixStream);

impl super::UnixStream for UnixStream {
    /// Create a connected pair of non-blocking Unix streams.
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

/// Future for an async read on the smarter `UnixStream`.
#[derive(Debug)]
pub struct Read<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r mut [u8],
}

impl Future for Read<'_> {
    type Output = Result<usize>;

    /// Attempt a non-blocking read, registering a wake-up on `WouldBlock`.
    ///
    /// If the read would block, [`wake_on_event`] spawns a background
    /// thread that waits for the FD to become readable and then calls
    /// `Waker::wake()` to unpark the executor.
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

/// Future for an async write on the smarter `UnixStream`.
#[derive(Debug)]
pub struct Write<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r [u8],
}

impl Future for Write<'_> {
    type Output = Result<usize>;

    /// Attempt a non-blocking write, registering a wake-up on `WouldBlock`.
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

/// Spawn a background thread that waits for `event` on `fd`, then wakes
/// the executor.
///
/// This is the "reactor" of this runtime: it bridges OS-level I/O readiness
/// notifications to the Rust `Waker` mechanism. The FD is cloned (via
/// [`AsFd::as_fd`] + [`try_clone_to_owned`](std::os::fd::BorrowedFd::try_clone_to_owned))
/// so the background thread owns its own handle and can outlive the borrow.
fn wake_on_event<Fd: AsFd>(fd: &Fd, event: Event, waker: &Waker) {
    let fd = fd.as_fd().try_clone_to_owned().unwrap();
    let waker = waker.clone();

    thread::spawn(move || {
        poll(&fd, event);

        waker.wake();
    });
}

/// Block the current thread until `fd` is ready for the given `event`.
///
/// Uses [`rustix::event::poll`] with no timeout (waits indefinitely).
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

/// The type of I/O event to wait for.
#[derive(Debug, Copy, Clone)]
enum Event {
    /// Wait until the FD has data available for reading (`POLLIN`).
    Read,
    /// Wait until the FD has buffer space for writing (`POLLOUT`).
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
