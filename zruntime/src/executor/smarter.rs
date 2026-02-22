//! Smarter executor — the working version.
//!
//! This fixes both problems of the [naive executor](super::naive):
//!
//! 1. **Real waker.** A [`ThreadWaker`] wraps the executor's
//!    [`Thread`] handle. When a future (or its reactor — see
//!    [`smarter::UnixStream`](crate::unix_stream::smarter)) calls
//!    `Waker::wake()`, the executor thread is unparked and re-polls
//!    the task.
//!
//! 2. **Concurrent task polling.** `run` polls **every** task in each
//!    iteration using [`retain_mut`](VecDeque::retain_mut), removing
//!    only those that are `Ready`. Tasks that are `Pending` stay in
//!    the queue and get another chance on the next iteration. This
//!    lets the writer task make progress while the reader is waiting.
//!
//! Between polling rounds the executor parks the thread, avoiding
//! busy-waiting. It is woken up when any future's reactor calls
//! `wake()`.

use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{channel, Receiver},
        Arc,
    },
    task::{Context, Poll, Wake},
    thread::{self, park, Thread},
};

use futures::pin_mut;

/// The smarter single-threaded executor.
///
/// See the [module-level documentation](self) for how it improves on the
/// naive version.
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

    /// Block the current thread on a single future, parking between polls.
    ///
    /// Unlike the naive version, the waker handed to the future is a real
    /// [`ThreadWaker`] that unparks this thread. When the future returns
    /// `Pending`, the thread parks instead of spinning, and will be woken
    /// when the future (or its I/O reactor) calls `wake()`.
    fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future,
    {
        pin_mut!(f);

        let waker = Arc::new(ThreadWaker(thread::current())).into();
        let mut cx = Context::from_waker(&waker);

        loop {
            match Pin::new(&mut f).poll(&mut cx) {
                Poll::Ready(val) => return val,
                Poll::Pending => {}
            }

            // Sleep until a waker unparks us.
            park();
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

    /// Poll **all** tasks in each iteration, removing completed ones.
    ///
    /// This is the key difference from the naive executor: instead of
    /// blocking on one task at a time, every task gets a chance to make
    /// progress in each round. After a round, if tasks remain, the
    /// executor parks until a waker unparks it.
    fn run(&mut self) {
        let waker = Arc::new(ThreadWaker(thread::current())).into();
        let mut cx = Context::from_waker(&waker);

        while !self.tasks.is_empty() {
            self.tasks
                .retain_mut(|task| match Pin::new(&mut task.future).poll(&mut cx) {
                    Poll::Ready(_) => false, // Task done, remove it.
                    Poll::Pending => true,   // Task still pending, keep it.
                });

            if !self.tasks.is_empty() {
                // Sleep until a reactor wakes us.
                park();
            }
        }
    }
}

/// A [`Wake`] implementation that unparks a specific thread.
///
/// This is the bridge between the I/O reactor (which detects that a file
/// descriptor is ready) and the executor (which needs to re-poll the
/// future). The reactor clones the `Waker` built from this struct; when
/// the FD event fires, calling `wake()` unparks the executor thread so
/// it can poll again.
struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
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
