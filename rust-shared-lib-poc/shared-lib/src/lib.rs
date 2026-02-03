//! Shared library that bundles serde, serde_json, and tokio.
//!
//! When built as a dylib, binaries linking against this library
//! share the same crate code at runtime, reducing total disk
//! and memory usage.

pub use serde;
pub use serde_json;
pub use tokio;
