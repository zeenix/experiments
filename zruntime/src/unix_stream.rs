//! Async Unix stream trait and implementations.
//!
//! A [`UnixStream`] wraps a non-blocking Unix domain socket and exposes
//! async `read`/`write` methods that return futures. Two implementations
//! are provided:
//!
//! - [`naive`]: Returns `Pending` on `WouldBlock` but **ignores the
//!   waker**, so the executor has no way to know when the FD becomes
//!   ready. Only works with a busy-polling executor (and even then,
//!   only if tasks are polled concurrently — which the naive executor
//!   doesn't do).
//! - [`smarter`]: On `WouldBlock`, spawns a background thread that
//!   uses [`poll(2)`](rustix::event::poll) to wait for the FD event
//!   and then calls `Waker::wake()`, properly integrating with the
//!   smarter executor's park/unpark mechanism.

use std::{future::Future, io::Result};

pub mod naive;
pub mod smarter;

/// An async Unix domain stream socket.
///
/// Implementations must use non-blocking I/O so that `read` and `write`
/// return futures instead of blocking the thread.
pub trait UnixStream: Sized {
    /// Create a connected pair of streams (like `socketpair(2)`).
    fn pipe() -> Result<(Self, Self)>;

    /// Asynchronously read bytes into `buf`, returning how many were read.
    fn read<'r>(&'r mut self, buf: &'r mut [u8]) -> impl Future<Output = Result<usize>> + 'r;

    /// Asynchronously write `buf` to the stream, returning how many bytes
    /// were written.
    fn write<'r>(&'r mut self, buf: &'r [u8]) -> impl Future<Output = Result<usize>> + 'r;
}
