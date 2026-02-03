# Rust Shared Library PoC

Demonstrates multiple Rust binaries sharing crate dependencies through a
`dylib` shared library with **native Rust API** - no wrappers, no FFI.

## Usage

The shared library re-exports crates. Binaries define their own types:

```rust
use shared_lib::serde::{Serialize, Deserialize};
use shared_lib::tokio;

#[derive(Serialize, Deserialize)]
#[serde(crate = "shared_lib::serde")]
struct MyConfig {
    name: String,
    value: i64,
}

#[tokio::main]
async fn main() {
    let config = MyConfig { name: "test".into(), value: 42 };
    println!("{}", shared_lib::serde_json::to_string(&config).unwrap());
}
```

## Binary Sizes

With dynamic linking, binaries are tiny since serde/tokio live in the shared lib:

| File | Size |
|------|------|
| bin-hello | 10K |
| bin-serialize | 26K |
| bin-async-task | 54K |
| libshared_lib.so | 1.2M |
| **Total** | **~1.3M** |

Compare to static linking where each binary would be ~1-2MB.

## Project Structure

```
rust-shared-lib-poc/
├── shared-lib/           # dylib - re-exports serde, tokio
│   ├── meson.build       # Meson build for shared lib
│   └── Cargo.toml        # Cargo build (standalone)
├── bin-hello/            # Example: serialization
├── bin-serialize/        # Example: serde with custom types
├── bin-async-task/       # Example: tokio async
├── meson.build           # Main meson build
├── Cargo.toml            # Cargo workspace
└── Cargo.lock            # Dependency lock file
```

## Building with Cargo

The `.cargo/config.toml` sets `-C prefer-dynamic`:

```bash
cargo build --release
```

## Building with Meson

Meson 1.5.0+ uses native Rust support with cargo wraps. Dependencies are
auto-resolved from Cargo.lock:

```bash
meson setup build
meson compile -C build
```

## Running

Set `LD_LIBRARY_PATH` to include the shared library and libstd:

```bash
RUSTUP_LIBDIR="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib"
export LD_LIBRARY_PATH="./build:$RUSTUP_LIBDIR"  # or ./target/release for cargo

./build/bin-hello
```

## Why dylib?

| Approach | Rust API | Toolchain | Trade-off |
|----------|----------|-----------|-----------|
| **dylib** | Native | Needs dynamic libstd | Best ergonomics |
| cdylib + abi_stable | Near-native (RString, RVec) | Standard | Wrapper types |
| cdylib + C FFI | Manual wrappers | Standard | Loses Rust ergonomics |

Since all systemd components would be built together with the same toolchain,
`dylib` is the right choice - ABI stability isn't needed.

## For systemd

If systemd builds Rust components with meson:
1. Use `-C prefer-dynamic` rustc flag
2. Ship `libsystemd_rust.so` (the shared lib) alongside binaries
3. Ensure libstd.so is available (dynamic linking requirement)

**Limitation:** Rust's `dylib` requires dynamic libstd - there's no way to
statically link std while sharing other crates. This is a rustc constraint,
not a build system limitation. LTO is also incompatible with dynamic linking.
