# ReDeFi Relay Node

## Project Description

Custom polkadot node for the ReDeFi runtime with EVM RPC support.

## Rust compiler versions

This release was built and tested against the following versions of rustc.

```
Rust Stable:  rustc 1.75.0 (82e1608df 2023-12-21)
```

Other versions may work.
Note: add targets:

```bash
rustup target add wasm32-unknown-unknown 
rustup target add x86_64-unknown-linux-musl
```

## Build

Build the node by cloning this repository and running the following commands from the root directory of the repo:

```bash
 cd polkadot
 cargo build --profile=production
```
