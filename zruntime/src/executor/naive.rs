use std::{
    future::Future,
    pin::Pin,
    sync::mpsc::{channel, Receiver},
    task::{Context, Poll},
};

use futures::pin_mut;

pub struct Executor {
    tasks: Vec<Task>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor { tasks: Vec::new() }
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

    pub fn spawn<F>(&mut self, future: F) -> TaskHandle<F::Output>
    where
        F: Future + Send + 'static,
    {
        let (sender, receiver) = channel();
        let future = Box::pin(async move {
            let res = future.await;
            sender.send(res).unwrap();
        });
        self.tasks.push(Task { future });

        TaskHandle { receiver }
    }

    pub fn run(&mut self) {
        while let Some(task) = self.tasks.pop() {
            self.block_on(task.future);
        }
    }
}

pub struct TaskHandle<Ret> {
    receiver: Receiver<Ret>,
}

impl<Ret> TaskHandle<Ret> {
    pub fn join(self) -> Ret {
        self.receiver.recv().unwrap()
    }
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}
