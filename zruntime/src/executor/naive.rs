use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::mpsc::{channel, Receiver},
    task::{Context, Poll},
};

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

    fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future,
    {
        futures::pin_mut!(f);

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        loop {
            match Pin::new(&mut f).poll(&mut cx) {
                Poll::Ready(val) => return val,
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

    fn run(&mut self) {
        while let Some(task) = self.tasks.pop_front() {
            self.block_on(task.future);
        }
    }
}

pub struct TaskHandle<Ret> {
    receiver: Receiver<Ret>,
}

impl<Ret> super::TaskHandle for TaskHandle<Ret> {
    type Output = Ret;

    fn join(self) -> Ret {
        self.receiver.recv().unwrap()
    }
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}
