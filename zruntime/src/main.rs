use std::{
    future::Future,
    pin::Pin,
    str::from_utf8,
    task::{Context, Poll},
};

mod executor;
mod unix_stream;

use executor::{naive, smarter, Executor, TaskHandle};
use unix_stream::{naive as naive_unix, smarter as smarter_unix, UnixStream};

struct MyFuture(u32);

impl Future for MyFuture {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0)
    }
}

async fn give_me_u32() -> u32 {
    MyFuture(42).await
}

fn main() {
    println!("Running smarter executor..");
    let (tx, rx) = smarter_unix::UnixStream::pipe().unwrap();
    run(smarter::Executor::new(), tx, rx);
    println!("");

    println!("Running naive executor..");
    let (tx, rx) = naive_unix::UnixStream::pipe().unwrap();
    run(naive::Executor::new(), tx, rx);
}

fn run<R, U>(mut executor: R, mut tx: U, mut rx: U)
where
    R: Executor,
    U: UnixStream + Send + Sync + 'static,
{
    let handle1 = executor.spawn(async move {
        let mut buf = [0; 50];
        let len = rx.read(&mut buf).await.unwrap();
        println!("\t{}", from_utf8(&buf[..len]).unwrap());
    });

    let handle2 = executor.spawn(async move {
        let msg = b"Hellllo! Jerry! Hellllo!";
        let written = tx.write(msg).await.unwrap();
        assert_eq!(written, msg.len());
    });

    executor.block_on(async move {});
    let num = executor.block_on(give_me_u32());
    println!("\tReceived number: {}", num);

    executor.run();

    handle1.join();
    handle2.join();
}
