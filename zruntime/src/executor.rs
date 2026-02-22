//! Executor traits and implementations.
//!
//! An executor is the component that drives futures to completion. It
//! repeatedly polls futures until they return `Poll::Ready`. Two
//! implementations are provided:
//!
//! - [`naive`]: A broken executor that busy-loops with a no-op waker and runs
//!   tasks one at a time. It cannot make progress when one task depends on
//!   another.
//! - [`smarter`]: A working executor that uses thread parking for efficient
//!   waiting and polls all tasks in each iteration, allowing concurrent
//!   progress.

use std::future::Future;

pub mod naive;
pub mod smarter;

/// A single-threaded async executor.
///
/// The executor owns a queue of spawned tasks and drives them to completion
/// when [`run`](Executor::run) is called.
pub trait Executor {
    /// Handle returned by [`spawn`](Executor::spawn) to retrieve the task's
    /// output.
    type TaskHandle<O>: TaskHandle<Output = O>;

    /// Run a single future to completion on the current thread.
    ///
    /// This blocks the calling thread until the future resolves.
    fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future;

    /// Enqueue a future for execution, returning a handle to its result.
    ///
    /// The future is not polled immediately — it will be driven to completion
    /// when [`run`](Executor::run) is called.
    fn spawn<F>(&mut self, future: F) -> Self::TaskHandle<F::Output>
    where
        F: Future + 'static;

    /// Drive all spawned tasks to completion.
    fn run(&mut self);
}

/// A handle to a spawned task's result.
pub trait TaskHandle {
    type Output;

    /// Block the calling thread until the task completes and return its
    /// output.
    fn join(self) -> Self::Output;
}
