# Substrate Node Template

> 原项目链接: https://github.com/substrate-developer-hub/substrate-node-template

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
Or for example:

```bash
rustfmt +nightly pallets/template/src/lib.rs
```