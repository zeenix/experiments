# Shared library that bundles serde, serde_json, and tokio
# Uses meson's native Rust support with automatic Cargo.lock parsing
#
# Dependencies are fetched from crates.io based on Cargo.lock at project root.
# Format: <package_name>-<api_version>-rs (e.g., serde-1-rs for serde 1.x)
#
# Features are resolved from Cargo.toml - meson reads the features specified
# in each crate's Cargo.toml to determine which features to enable.
#
# Uses dylib crate type which provides Rust metadata and allows
# binaries to link dynamically. Requires libstd.so at runtime.

# Dependencies auto-resolved from Cargo.lock
# Features come from shared-lib/Cargo.toml
#
# IMPORTANT: Request tokio first! tokio-macros requires proc-macro2 with
# 'default' feature. If serde_derive is processed first, proc-macro2 gets
# configured without 'default', causing a feature conflict later.
tokio_dep = dependency('tokio-1-rs')
serde_dep = dependency('serde-1-rs')
serde_derive_dep = dependency('serde_derive-1-rs')
serde_json_dep = dependency('serde_json-1-rs')

shared_lib = shared_library('shared_lib',
  'src/lib.rs',
  rust_abi: 'rust',
  rust_args: ['-C', 'prefer-dynamic'],
  dependencies: [serde_dep, serde_derive_dep, serde_json_dep, tokio_dep],
  install: true,
)

shared_lib_dep = declare_dependency(
  link_with: shared_lib,
  dependencies: [serde_dep, serde_derive_dep, serde_json_dep, tokio_dep],
)

meson.override_dependency('shared-lib-0.1-rs', shared_lib_dep)
