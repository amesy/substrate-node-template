# Substrate Node Template

[![Try on playground](https://img.shields.io/badge/Playground-Node_Template-brightgreen?logo=Parity%20Substrate)](https://docs.substrate.io/playground/) [![Matrix](https://img.shields.io/matrix/substrate-technical:matrix.org)](https://matrix.to/#/#substrate-technical:matrix.org)

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