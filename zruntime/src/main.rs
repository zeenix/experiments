use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{sync_channel, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

mod executor;

use executor::naive;

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
    let mut executor = naive::Executor::new();

    let handle = executor.spawn(async {
        println!("Hello from the future!");
    });

    executor.block_on(async {
        println!("Hello from the executor!");
    });
    let num = executor.block_on(give_me_u32());
    println!("Received number: {}", num);

    handle.join();
}
