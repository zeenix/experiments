use std::str::from_utf8;

mod executor;
mod unix_stream;

use executor::{naive, smarter, Executor, TaskHandle};
use unix_stream::{naive as naive_unix, smarter as smarter_unix, UnixStream};

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

    executor.run();

    handle1.join();
    handle2.join();
}
