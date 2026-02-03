//! Serialization demo using the shared library.
//!
//! Uses serde and serde_json from the shared library dylib at runtime.

use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ServiceUnit {
    name: String,
    #[serde(rename = "type")]
    unit_type: String,
    dependencies: Vec<String>,
}

fn main() {
    println!("=== Serialization Binary ===\n");

    let unit = ServiceUnit {
        name: "nginx.service".into(),
        unit_type: "simple".into(),
        dependencies: vec!["network.target".into(), "remote-fs.target".into()],
    };

    println!("ServiceUnit as JSON:");
    println!("{}\n", serde_json::to_string_pretty(&unit).unwrap());

    // Deserialization
    let json = r#"{"name":"parsed.service","type":"oneshot","dependencies":[]}"#;
    let parsed: ServiceUnit = serde_json::from_str(json).unwrap();
    println!("Parsed: {:?}", parsed);
}
