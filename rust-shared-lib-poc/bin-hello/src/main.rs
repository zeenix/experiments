//! Simple hello binary demonstrating shared crate usage.
//!
//! Uses serde and serde_json from the shared library dylib at runtime.

use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    value: i64,
    enabled: bool,
}

fn main() {
    println!("=== Hello Binary ===\n");

    let config = Config {
        name: "greeting".into(),
        value: 1,
        enabled: true,
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("Config as JSON:\n{}", json);
}
