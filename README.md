# Substrate Node Template

## Getting Started

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/node-template --dev
```

### Test

```bash
cargo test -p  PALLETS-PACKAGE-NAME
```

### Format

```bash

cargo +nightly fmt
```
or for example:

```bash
rustfmt +nightly pallets/template/src/lib.rs
```