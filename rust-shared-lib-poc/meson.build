# Rust Shared Library PoC - Meson Build
#
# Demonstrates multiple binaries sharing crate dependencies through a dylib.
# Uses meson's native Rust support with automatic Cargo.lock parsing (since 1.5.0).
#
# Meson automatically reads Cargo.lock and fetches crates from crates.io.
# Dependencies use format: <package_name>-<api_version>-rs
#
# Features are resolved from each crate's Cargo.toml - meson reads the features
# specified there to determine which features to enable for dependencies.
#
# Note: Requires libstd.so at runtime. See STATIC_STD_INVESTIGATION.md for
# analysis of approaches to embed std statically (spoiler: not currently possible).

project('rust-shared-lib-poc', 'rust',
  version: '0.1.0',
  meson_version: '>= 1.5.0',
  default_options: [
    'rust_std=2021',
  ],
)

# Build the shared library (bundles serde, serde_json, tokio)
subdir('shared-lib')

# Get dependencies for binaries (shared with shared_lib)
serde_dep = dependency('serde-1-rs')
serde_derive_dep = dependency('serde_derive-1-rs')
serde_json_dep = dependency('serde_json-1-rs')
tokio_dep = dependency('tokio-1-rs')

# Build binaries linking against the shared library
# Binaries use the same crate instances as shared_lib for trait compatibility
executable('bin-hello',
  'bin-hello/src/main.rs',
  dependencies: [shared_lib_dep, serde_dep, serde_derive_dep, serde_json_dep],
  rust_args: ['-C', 'prefer-dynamic'],
  install: true,
)

executable('bin-serialize',
  'bin-serialize/src/main.rs',
  dependencies: [shared_lib_dep, serde_dep, serde_derive_dep, serde_json_dep],
  rust_args: ['-C', 'prefer-dynamic'],
  install: true,
)

executable('bin-async-task',
  'bin-async-task/src/main.rs',
  dependencies: [shared_lib_dep, tokio_dep],
  rust_args: ['-C', 'prefer-dynamic'],
  install: true,
)
