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

pub struct Executor {
    receiver: Receiver<Arc<Task>>,
}

pub struct Spawner {
    sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn<F>(&self, f: F)
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

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).unwrap();
    }
}

impl Executor {
    pub fn new() -> (Executor, Spawner) {
        let (sender, receiver) = sync_channel(0);
        (Executor { receiver }, Spawner { sender })
    }

    pub fn block_on<F>(&mut self, f: F) -> F::Output
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
