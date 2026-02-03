//! Async task demo using the shared library.

use std::time::Duration;
use shared_lib::tokio;

#[tokio::main]
async fn main() {
    println!("=== Async Task Binary ===\n");

    // Spawn concurrent tasks
    let h1 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        "Task 1 done"
    });
    let h2 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        "Task 2 done"
    });

    let (r1, r2) = tokio::join!(h1, h2);
    println!("{}", r1.unwrap());
    println!("{}", r2.unwrap());
}
