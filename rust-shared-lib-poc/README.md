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

## Building

Standard Rust toolchains ship dynamic libstd (e.g., at
`~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd-*.so`).

The `.cargo/config.toml` sets `-C prefer-dynamic`, so just run:

```bash
cargo build --release
```

To run, set `LD_LIBRARY_PATH` to include both the shared library and libstd:

```bash
RUSTUP_LIBDIR="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib"
export LD_LIBRARY_PATH="$(pwd)/target/release:$RUSTUP_LIBDIR"

./target/release/bin-hello
./target/release/bin-serialize
./target/release/bin-async-task
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
├── bin-hello/            # Example: serialization
├── bin-serialize/        # Example: serde with custom types
├── bin-async-task/       # Example: tokio async
└── README.md
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

If systemd builds Rust components, the build system should:
1. Build/use Rust with `--enable-shared`
2. Install `libsystemd_rust.so` alongside binaries
3. Binaries link against it automatically via cargo

No wrapper code, no FFI, just normal Rust.
