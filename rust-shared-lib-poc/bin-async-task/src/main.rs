//! Async task demo using the shared library.
//!
//! Uses tokio from the shared library dylib at runtime.

use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("=== Async Task Binary ===\n");

    let h1 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        "Task 1 completed"
    });

    let h2 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        "Task 2 completed"
    });

    let (r1, r2) = tokio::join!(h1, h2);
    println!("{}", r1.unwrap());
    println!("{}", r2.unwrap());
}
