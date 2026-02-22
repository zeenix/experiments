//! Naive executor — the broken first attempt.
//!
//! This executor has two fundamental flaws that cause it to hang:
//!
//! 1. **No-op waker.** `block_on` creates a [`Context`] with
//!    [`noop_waker`](futures::task::noop_waker_ref), so when a future returns
//!    [`Poll::Pending`], nothing ever wakes the executor to retry. The loop
//!    just spins, re-polling the same future forever.
//!
//! 2. **Sequential task processing.** `run` pops tasks one at a time and
//!    calls `block_on` on each. If the first task is waiting for data that
//!    the second task would produce, `block_on` never returns and the second
//!    task never runs.
//!
//! Together these make the executor hang whenever tasks depend on each other
//! (e.g. a reader waiting for a writer).

use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::mpsc::{channel, Receiver},
    task::{Context, Poll},
};

/// The naive single-threaded executor.
///
/// See the [module-level documentation](self) for why this doesn't work.
pub struct Executor {
    tasks: VecDeque<Task>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: VecDeque::new(),
        }
    }
}

impl super::Executor for Executor {
    type TaskHandle<O> = TaskHandle<O>;

    /// Busy-loop a future to completion using a no-op waker.
    ///
    /// Because the waker does nothing, futures that return `Pending` and rely
    /// on being woken (i.e. all I/O futures) will cause this to spin forever.
    fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future,
    {
        futures::pin_mut!(f);

        // A no-op waker: calling `wake()` on it does absolutely nothing.
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        loop {
            match Pin::new(&mut f).poll(&mut cx) {
                Poll::Ready(val) => return val,
                // Future is not ready, but nobody will wake us — just
                // spin and poll again immediately.
                Poll::Pending => {}
            }
        }
    }

    fn spawn<F>(&mut self, future: F) -> TaskHandle<F::Output>
    where
        F: Future + 'static,
    {
        let (sender, receiver) = channel();
        let future = Box::pin(async move {
            let res = future.await;
            sender.send(res).unwrap();
        });
        self.tasks.push_back(Task { future });

        TaskHandle { receiver }
    }

    /// Run tasks **sequentially**: pop the first task, block on it until
    /// done, then move to the next.
    ///
    /// This means task N+1 cannot run until task N completes. If task N is
    /// waiting for I/O that task N+1 would produce, we deadlock.
    fn run(&mut self) {
        while let Some(task) = self.tasks.pop_front() {
            self.block_on(task.future);
        }
    }
}

/// Handle to a spawned task's result, backed by an MPSC channel.
pub struct TaskHandle<Ret> {
    receiver: Receiver<Ret>,
}

impl<Ret> super::TaskHandle for TaskHandle<Ret> {
    type Output = Ret;

    fn join(self) -> Ret {
        self.receiver.recv().unwrap()
    }
}

/// An opaque task: a boxed, pinned, type-erased future.
struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}
