# alloy-transport-window

Alloy transport and signer implementations for browser-based Ethereum wallets via `window.ethereum` (EIP-1193).

This crate provides a bridge between [Alloy](https://github.com/alloy-rs/alloy) and browser wallet extensions like MetaMask, Rabby, and Coinbase Wallet that implement the EIP-1193 standard.

## Features

- **WindowTransport**: Routes Alloy RPC calls through `window.ethereum`
- **WindowSigner**: Delegates message signing to the browser wallet
- **Transaction Support**: Send transactions via `eth_sendTransaction` with wallet-side signing
- **WASM-first**: Built specifically for browser environments
- **Lightweight**: ~300 lines of well-documented code

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

## Usage

### Connecting to a Wallet

```rust
use alloy::signers::Signer;
use alloy_transport_window::WindowSigner;

// Trigger wallet connection prompt
let signer = WindowSigner::new().await?;
let address = signer.address();
println!("Connected: {}", address);

// Get existing connection without prompting
let signer = WindowSigner::from_existing().await?;
```

### Reading Blockchain Data

No wallet connection needed for read-only operations:

```rust
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy_transport_window::WindowTransport;

// Create provider
let transport = WindowTransport::new()?;
let client = RpcClient::new(transport, false);
let provider = ProviderBuilder::new().connect_client(client);

// Query blockchain
let block_number = provider.get_block_number().await?;
let chain_id = provider.get_chain_id().await?;
let balance = provider.get_balance(address).await?;
```

### Sending Transactions

Browser wallets handle signing internally via `eth_sendTransaction`:

```rust
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::Signer;
use alloy_transport_window::{WindowSigner, WindowTransport};

// Connect wallet
let signer = WindowSigner::new().await?;
let from = signer.address();

// Create provider
let transport = WindowTransport::new()?;
let provider = ProviderBuilder::new().connect_client(RpcClient::new(transport, false));

// Build transaction
let tx = TransactionRequest::default()
    .with_from(from)
    .with_to("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse::<Address>()?)
    .with_value(U256::from(1_000_000_000_000_000_000_u64)); // 1 ETH

// Send (triggers wallet approval prompt)
let pending = provider.send_transaction(tx).await?;
let receipt = pending.get_receipt().await?;
```

### Smart Contract Calls

Read from contracts using `sol!` macro and type-safe bindings:

```rust
use alloy::sol;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy_transport_window::WindowTransport;

// Define contract interface
sol! {
    #[sol(rpc)]
    IPool,
    "path/to/abi.json"
}

// Create provider
let transport = WindowTransport::new()?;
let provider = ProviderBuilder::new().connect_client(RpcClient::new(transport, false));

// Create contract instance
let pool = IPool::new(pool_address, &provider);

// Call view function
let data = pool.getUserAccountData(user_address).call().await?;
```

### Message Signing

The `WindowSigner` implements Alloy's `Signer` trait:

```rust
use alloy::signers::Signer;
use alloy_transport_window::WindowSigner;

let signer = WindowSigner::new().await?;

// Sign a message (uses personal_sign)
let message = b"Hello, Ethereum!";
let signature = signer.sign_message(message).await?;

// Sign a hash (uses eth_sign)
let hash = b256!("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
let signature = signer.sign_hash(&hash).await?;
```

## How It Works

### Transaction Flow

Browser wallets use `eth_sendTransaction` which signs and broadcasts in one call. Unlike local signers, they don't expose private keys or allow pre-signing transactions.

**Architecture:**
- `WindowSigner` implements `Signer` for message signing only
- `WindowSigner` does NOT implement `TxSigner` or `NetworkWallet`
- Use `provider.send_transaction()` directly - no wallet attachment needed
- The `WindowTransport` automatically routes transactions to the browser wallet
- Users approve each transaction through their wallet UI

### Browser Requirements

This crate requires:
- A Web3-compatible browser wallet extension installed
- JavaScript enabled
- `window.ethereum` object available

Check for wallet availability:

```rust
use alloy_transport_window::{WindowError, WindowTransport};

match WindowTransport::new() {
    Ok(transport) => {
        // Wallet available
    }
    Err(WindowError::NoWallet) => {
        // Show installation instructions
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Error Handling

The crate provides specific error variants:

```rust
use alloy_transport_window::{WindowError, WindowSigner};

match WindowSigner::new().await {
    Ok(signer) => { /* connected */ }
    Err(WindowError::NoWallet) => {
        // No wallet installed
    }
    Err(WindowError::UserRejected) => {
        // User rejected connection
    }
    Err(WindowError::Rpc(msg)) => {
        // RPC error from wallet
    }
    Err(e) => {
        // Other error
    }
}
```

## Complete Examples

See the [example/](example/) directory for a full Dioxus web app demonstrating:
- [Wallet connection with balance display](example/src/examples/wallet_connect.rs)
- [Sending ETH transactions](example/src/examples/send_transaction.rs)
- [Querying blockchain data](example/src/examples/block_explorer.rs)
- [Calling Aave contracts](example/src/examples/aave_pool.rs)

## Supported Wallets

Any browser wallet implementing EIP-1193 should work:

- MetaMask
- Rabby
- Coinbase Wallet
- Rainbow
- Trust Wallet
- Brave Wallet

## Security

- Private keys never leave the browser wallet
- All signing operations require user approval
- User rejection is properly detected and handled
- No direct key access or transaction pre-signing

## MSRV

Minimum Supported Rust Version: 1.88

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
