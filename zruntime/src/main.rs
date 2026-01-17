use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{sync_channel, Receiver, Sender, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

use futures::{future::BoxFuture, pin_mut, task::ArcWake, FutureExt};

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
    (Executor { receiver }, Spawner { sender })
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).unwrap();
    }
}

impl Executor {
    fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future,
    {
        pin_mut!(f);

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        loop {
            match Pin::new(&mut f).poll(&mut cx) {
                Poll::Ready(val) => return val,
                Poll::Pending => {}
            }
        }
    }
}

fn main() {
    let (mut executor, _spawner) = create_executor();

    /*spawner.spawn(async {
        println!("Hello from the future!");
    });*/

    executor.block_on(async {
        println!("Hello from the executor!");
    });
    let num = executor.block_on(give_me_u32());
    println!("Received number: {}", num);
}
