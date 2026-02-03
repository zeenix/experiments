//! Simple hello binary demonstrating shared crate usage.

use shared_lib::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "shared_lib::serde")]
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

    let json = shared_lib::serde_json::to_string_pretty(&config).unwrap();
    println!("Config as JSON:\n{}", json);
}
