use std::{future::Future, pin::Pin, sync::{Arc, Mutex, mpsc::{Receiver, Sender, SyncSender, sync_channel}}, task::{Context, Poll}};

use futures::{FutureExt, future::BoxFuture, task::ArcWake};

struct MyFuture(u32);

impl Future for MyFuture {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0)
    }
}

async fn give_me_u32() -> u32 {
    MyFuture(42).await
}

struct Executor {
    receiver: Receiver<Arc<Task>>,
}
struct Spawner {
    sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn<F>(&self, f: F) 
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let future = Mutex::new(Some(f.boxed()));
        let task = Arc::new(Task {
            future,
            sender: self.sender.clone(),
        });

        self.sender.send(task).unwrap();
    }
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    sender: SyncSender<Arc<Task>>,
}

fn create_executor() -> (Executor, Spawner) {
    let (sender, receiver) = sync_channel(0);
    (
        Executor { receiver },
        Spawner { sender },
    )
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).unwrap();
    }
}

impl Executor {
    fn block_on<F>(f: F) -> F::Output
    where
        F: Future,
    {
        // 
        //
        // We were here!
        //
        //
        let pin = Pin::new(&mut f);

        let cx = Context::from_waker(std::task::noop_waker_ref());

        pin.poll(&mut cx).expect("poll failed");
    }
}

fn main() {

}
