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
cargo test -p  pallet-poe
```

## Other operations

### Format

```bash

cargo +nightly fmt
```
or for example:

```bash
rustfmt +nightly pallets/template/src/lib.rs
```

### Generate metadata.json

```bash
cargo install subxt-cli
subxt metadata --url http://127.0.0.1:9933 --format json > metadata.json
```

### Macro expand

```bash
cd pallets/poe
cargo expand > expand.rs
```