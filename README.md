# alloy-transport-window

Alloy transport and signer implementations for browser-based Ethereum wallets via `window.ethereum` (EIP-1193).

This crate provides a bridge between [Alloy](https://github.com/alloy-rs/alloy) and browser wallet extensions like MetaMask, Rabby, and Coinbase Wallet that implement the EIP-1193 standard.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
alloy-transport-window = "0.1"
alloy = { version = "1.4", default-features = false, features = ["contract", "signers", "rpc-types"] }
```

Since this is a WASM-only crate, you'll need to target `wasm32-unknown-unknown`:

```bash
rustup target add wasm32-unknown-unknown
```

## Complete Examples

See the [example/](example/) directory for a full Dioxus web app demonstrating:
- [Wallet connection with balance display](example/src/examples/wallet_connect.rs)
- [Sending ETH transactions](example/src/examples/send_transaction.rs)
- [Querying blockchain data](example/src/examples/block_explorer.rs)
- [Calling Aave contracts](example/src/examples/aave_pool.rs)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
