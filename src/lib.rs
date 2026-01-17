//! # alloy-transport-window
//!
//! Alloy transport and signer implementations for browser wallets via `window.ethereum` (EIP-1193).
//!
//! This crate provides minimal, zero-dependency bridge between Alloy and browser-based Web3 wallets
//! like MetaMask, Rabby, Coinbase Wallet, etc.
//!
//! ## Features
//!
//! - **WindowTransport**: Implements Alloy's `Transport` trait to route RPC calls through `window.ethereum`
//! - **WindowSigner**: Implements Alloy's `Signer` trait for message signing (note: NOT for transaction signing)
//! - **WASM Compatible**: Designed specifically for use in browser environments
//! - **Transaction Support**: Send transactions via `eth_sendTransaction` - browser wallet handles signing
//! - **Minimal Code**: ~200 lines of well-documented code
//!
//! ## Example - Read-only Provider
//!
//! ```rust,ignore
//! use alloy_provider::ProviderBuilder;
//! use alloy_rpc_client::RpcClient;
//! use alloy_transport_window::WindowTransport;
//!
//! // Create transport from window.ethereum
//! let transport = WindowTransport::new()?;
//!
//! // Create RPC client and provider
//! let client = RpcClient::new(transport, false);
//! let provider = ProviderBuilder::new().connect_client(client);
//!
//! // Get block number
//! let block = provider.get_block_number().await?;
//! ```
//!
//! ## Example - Sending Transactions
//!
//! ```rust,ignore
//! use alloy_network::TransactionBuilder;
//! use alloy_provider::{Provider, ProviderBuilder};
//! use alloy_rpc_client::RpcClient;
//! use alloy_rpc_types::TransactionRequest;
//! use alloy_transport_window::{WindowTransport, WindowSigner};
//!
//! // Request wallet access to get the address
//! let signer = WindowSigner::new().await?;
//! let from_address = signer.address();
//!
//! // Create provider
//! let transport = WindowTransport::new()?;
//! let client = RpcClient::new(transport, false);
//! let provider = ProviderBuilder::new().connect_client(client);
//!
//! // Send a transaction - browser wallet will prompt user to sign
//! let tx = TransactionRequest::default()
//!     .with_from(from_address)
//!     .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
//!     .with_value(U256::from(100));
//! let pending = provider.send_transaction(tx).await?;
//! let receipt = pending.get_receipt().await?;
//! ```
//!
//! ## Note on Transaction Signing
//!
//! Browser wallets use `eth_sendTransaction` which signs AND broadcasts transactions in a single call.
//! They don't support `eth_sign` for security reasons. Therefore:
//! - `WindowSigner` implements `Signer` for message signing (personal_sign)
//! - `WindowSigner` does NOT implement `TxSigner` or `NetworkWallet`
//! - To send transactions, use `provider.send_transaction()` directly (no wallet attachment needed)
//! - The `WindowTransport` automatically routes transaction requests through the browser wallet

mod error;
mod signer;
mod transport;

pub use error::{Result, WindowError};
pub use signer::WindowSigner;
pub use transport::WindowTransport;
