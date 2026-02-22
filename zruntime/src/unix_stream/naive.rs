//! Naive async Unix stream — ignores the waker.
//!
//! This implementation wraps a non-blocking [`UnixStream`](unix::net::UnixStream)
//! and implements `Future` for read/write operations. When the underlying
//! syscall returns `WouldBlock`, the future returns `Poll::Pending` — but
//! **discards the [`Context`]** (note the `_cx` parameter). This means:
//!
//! - No reactor ever calls `Waker::wake()`.
//! - The executor has no signal that the FD has become ready.
//! - Progress can only be made if the executor happens to re-poll the
//!   future — which requires either busy-looping or an external event.
//!
//! Combined with the [naive executor](crate::executor::naive) (which
//! processes tasks sequentially and busy-loops with a no-op waker), this
//! results in a deadlock whenever the reader is polled before the writer.

use std::{
    future::Future,
    io::{self, Result},
    os::unix,
    pin::Pin,
    task::{Context, Poll},
};

/// A non-blocking Unix domain stream socket (naive version).
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

/// Future for an async read on the naive `UnixStream`.
#[derive(Debug)]
pub struct Read<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r mut [u8],
}

impl Future for Read<'_> {
    type Output = Result<usize>;

    /// Attempt a non-blocking read.
    ///
    /// Returns `Ready(Ok(n))` if data was read, `Ready(Err(..))` on real
    /// errors, or `Pending` on `WouldBlock`. Note that `_cx` is ignored —
    /// no waker is registered, so nobody will wake the executor when data
    /// arrives.
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::Read;

        let this = self.get_mut();
        match this.stream.read(this.buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

/// Future for an async write on the naive `UnixStream`.
#[derive(Debug)]
pub struct Write<'r> {
    stream: &'r mut unix::net::UnixStream,
    buf: &'r [u8],
}

impl Future for Write<'_> {
    type Output = Result<usize>;

    /// Attempt a non-blocking write.
    ///
    /// Same caveat as [`Read::poll`]: the waker is ignored.
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::io::Write;

        let this = self.get_mut();
        match this.stream.write(this.buf) {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
