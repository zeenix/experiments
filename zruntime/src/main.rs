//! # zruntime: An Educational Async Runtime
//!
//! This crate is a minimal async runtime built from scratch to demonstrate how
//! async/await works under the hood in Rust. It accompanies a talk that
//! progressively builds up the runtime in two stages:
//!
//! ## Stage 1: Naive (broken)
//!
//! A minimal executor and I/O layer that compiles but **hangs at runtime**.
//! The naive executor busy-loops with a no-op waker and processes tasks
//! sequentially, while the naive `UnixStream` ignores the `Waker` entirely.
//! When the reader task is polled first and the data hasn't been written yet,
//! `block_on` spins forever because nothing can wake it — the writer task
//! never gets a chance to run.
//!
//! ## Stage 2: Smarter (working)
//!
//! Fixes both problems:
//!
//! - The executor uses a `ThreadWaker` that
//!   parks/unparks the executor thread, and `run()` polls **all** tasks in
//!   each iteration (not sequentially), so tasks can make progress while
//!   others wait.
//! - The `UnixStream` futures spawn a background thread that calls
//!   [`poll(2)`](rustix::event::poll) on the file descriptor and invokes
//!   `Waker::wake()` when the FD is ready, which unparks the executor.
//!
//! ## Running
//!
//! ```text
//! $ cargo run
//! Running smarter executor..
//!     Message from Uncle Leo: Hellllo! Jerry! Hellllo!
//!
//! Running naive executor..
//!     <hangs forever>
//! ```
//!
//! The smarter executor runs first (and succeeds), then the naive executor
//! runs and hangs — demonstrating exactly why proper waking matters.

use std::str::from_utf8;

mod executor;
mod unix_stream;

use executor::{naive, smarter, Executor, TaskHandle};
use unix_stream::{naive as naive_unix, smarter as smarter_unix, UnixStream};

fn main() {
    // Run the smarter (working) version first so we see output before the
    // naive version hangs.
    println!("Running smarter executor..");
    let (tx, rx) = smarter_unix::UnixStream::pipe().unwrap();
    run(smarter::Executor::new(), tx, rx);
    println!("");

    // The naive version will hang because the reader task busy-loops on a
    // no-op waker and the writer never gets scheduled.
    println!("Running naive executor..");
    let (tx, rx) = naive_unix::UnixStream::pipe().unwrap();
    run(naive::Executor::new(), tx, rx);
}

/// Spawn a reader and a writer task on the given executor and run them.
///
/// This is the test scenario used by both executor implementations: one task
/// writes a message to a Unix stream while another reads it. With the smarter
/// executor both tasks get polled in each iteration, so the writer makes
/// progress while the reader is pending. With the naive executor, `block_on`
/// is called on the reader first, which never completes because the writer
/// hasn't run yet.
fn run<R, U>(mut executor: R, mut tx: U, mut rx: U)
where
    R: Executor,
    U: UnixStream + Send + Sync + 'static,
{
    // Task 1: read from the stream and print the message.
    let handle1 = executor.spawn(async move {
        let mut buf = [0; 50];
        let len = rx.read(&mut buf).await.unwrap();
        println!(
            "\tMessage from Uncle Leo: {}",
            from_utf8(&buf[..len]).unwrap()
        );
    });

    // Task 2: write a message to the stream.
    let handle2 = executor.spawn(async move {
        let msg = b"Hellllo! Jerry! Hellllo!";
        let written = tx.write(msg).await.unwrap();
        assert_eq!(written, msg.len());
    });

    executor.run();

    handle1.join();
    handle2.join();
}
