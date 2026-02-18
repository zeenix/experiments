use std::future::Future;

pub mod naive;
pub mod smarter;

pub trait Executor {
    type TaskHandle<O>: TaskHandle<Output = O>;

    fn block_on<F>(&mut self, f: F) -> F::Output
    where
        F: Future;

    fn spawn<F>(&mut self, future: F) -> Self::TaskHandle<F::Output>
    where
        F: Future + 'static;

    fn run(&mut self);
}

pub trait TaskHandle {
    type Output;

    fn join(self) -> Self::Output;
}
