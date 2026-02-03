# Investigation: Static std in Shared Library

## Goal
Embed Rust's standard library statically into the shared library so binaries don't need a separate `libstd.so`.

## Approaches Tried

### 1. cdylib with static std (doesn't work)
```toml
[lib]
crate-type = ["cdylib"]
```

**Result**: cdylib statically links std, but doesn't provide Rust metadata. Binaries cannot use the Rust API.

### 2. cdylib + rlib dual build (doesn't work)
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

**Result**: Cargo prefers rlib for linking, defeating the purpose. Binaries statically link the rlib.

### 3. Hollow rlib (metadata only) + cdylib (doesn't work)
Create rlib with metadata stripped of code, then link against cdylib.

**Result**: Rustc requires actual code in rlib format, not just metadata. Cannot compile.

### 4. Version script to export all symbols from cdylib (partial)
Using `--version-script` to export mangled Rust symbols:
```
{ global: *; };
```

**Result**: Exports 1947 symbols including std runtime, but binaries still need metadata to compile against.

### 5. no_std binaries (not practical)
Build binaries as no_std and link against cdylib.

**Result**: Would require rewriting all code to be no_std compatible. Not practical.

## Root Cause

Rust's compilation model has a fundamental constraint:
1. **Compilation** requires metadata (types, traits, generics) from `.rmeta`/`.rlib`/`.dylib`
2. **cdylib** only provides code, no metadata (designed for C FFI)
3. **dylib** provides both metadata and code, but requires dynamic libstd

There's no mechanism to say "use this library for metadata, but link against that library for code."

## Current Working Solution

Use `dylib` crate type with dynamic libstd:

```toml
[lib]
crate-type = ["dylib"]
```

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "prefer-dynamic"]
```

### Results
- bin-hello: 10KB
- bin-serialize: 26KB
- bin-async-task: 55KB
- libshared_lib.so: 1.2MB
- libstd.so: 9.7MB (stripped)

### Size Analysis

| Scenario | Static (LTO) | Dynamic (bundled libstd) | Dynamic (system libstd) |
|----------|--------------|--------------------------|-------------------------|
| 1 binary | 294KB | 10.9MB | 1.2MB |
| 5 binaries | 1.5MB | 11.0MB | 1.4MB |
| 10 binaries | 2.9MB | 11.2MB | 1.5MB |
| 50 binaries | 14.7MB | 12.4MB | 2.7MB |

**Break-even points:**
- With bundled libstd: ~41 binaries
- With system libstd: ~5 binaries

## Recommendations

### Option A: Accept dynamic libstd (smallest binaries)
- Use dylib approach as-is
- Rely on system-provided libstd.so (same as any Rust dynamic linking)
- Best for distros with stable Rust toolchain packaging

### Option B: Bundle libstd.so with version control
- Ship libstd.so alongside binaries
- Use rpath to ensure binaries find the bundled version
- Avoids ABI compatibility issues with system libstd
- Makes sense for 40+ binaries

### Option C: Static linking with LTO (simplest)
- Just use static linking with LTO
- 294KB per binary
- Best for <40 binaries
- No runtime dependencies

## Potential Future Solutions

1. **Rust RFC for "metadata-only" crates**: Allow specifying metadata source separately from link source
2. **Custom linking via Meson**: Post-processing to merge cdylib symbols with binary
3. **Symbol versioning**: Export cdylib symbols with versioned names that binaries link against
