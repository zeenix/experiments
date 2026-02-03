//! Serialization demo using the shared library.

use shared_lib::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "shared_lib::serde")]
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
    println!("{}\n", shared_lib::serde_json::to_string_pretty(&unit).unwrap());

    // Deserialization
    let json = r#"{"name":"parsed.service","type":"oneshot","dependencies":[]}"#;
    let parsed: ServiceUnit = shared_lib::serde_json::from_str(json).unwrap();
    println!("Parsed: {:?}", parsed);
}
