//! Send Transaction Example
//!
//! Demonstrates sending transactions via browser wallet.
//! Shows how to:
//! - Build transaction requests with TransactionRequest
//! - Send transactions through the provider (delegates to eth_sendTransaction)
//! - Wait for transaction confirmation
//! - Handle transaction errors
//!
//! Note: Browser wallets use eth_sendTransaction which signs AND sends in one call.
//! You don't attach a wallet - the WindowTransport routes transactions through the browser wallet automatically.

use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::Signer;
use alloy_transport_window::{WindowSigner, WindowTransport};
use dioxus::logger::tracing;
use dioxus::prelude::*;

#[component]
pub fn SendTransaction() -> Element {
    let mut wallet_address = use_signal(|| Option::<Address>::None);
    let mut recipient = use_signal(|| String::from(""));
    let mut amount = use_signal(|| String::from("0.001"));
    let mut tx_hash = use_signal(|| Option::<String>::None);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::from("Not connected"));
    let mut is_sending = use_signal(|| false);

    // Connect wallet
    let connect_wallet = move |_| {
        spawn(async move {
            status_msg.set("Connecting to wallet...".to_string());
            error_msg.set(None);

            match WindowSigner::new().await {
                Ok(signer) => {
                    let addr = signer.address();
                    wallet_address.set(Some(addr));
                    status_msg.set("Wallet connected".to_string());
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to connect: {}", e)));
                    status_msg.set("Connection failed".to_string());
                }
            }
        });
    };

    // Send transaction
    let send_tx = move |_| {
        spawn(async move {
            is_sending.set(true);
            tx_hash.set(None);
            error_msg.set(None);
            status_msg.set("Preparing transaction...".to_string());

            // Validate wallet
            let wallet_addr = match wallet_address() {
                Some(a) => a,
                None => {
                    error_msg.set(Some("Please connect wallet first".to_string()));
                    status_msg.set("Error".to_string());
                    is_sending.set(false);
                    return;
                }
            };

            // Parse recipient address
            let to_addr = match recipient().parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    error_msg.set(Some(format!("Invalid recipient address: {}", e)));
                    status_msg.set("Error".to_string());
                    is_sending.set(false);
                    return;
                }
            };

            // Parse amount (convert ETH to wei)
            let amount_eth = match amount().parse::<f64>() {
                Ok(a) => a,
                Err(e) => {
                    error_msg.set(Some(format!("Invalid amount: {}", e)));
                    status_msg.set("Error".to_string());
                    is_sending.set(false);
                    return;
                }
            };

            // Convert to wei (1 ETH = 10^18 wei)
            let amount_wei = U256::from((amount_eth * 1_000_000_000_000_000_000.0) as u128);

            tracing::info!(
                "Preparing to send {} wei from {} to {}",
                amount_wei,
                wallet_addr,
                to_addr
            );

            // Create provider (no wallet attachment needed - browser handles signing)
            let transport = match WindowTransport::new() {
                Ok(t) => t,
                Err(e) => {
                    error_msg.set(Some(format!("Transport error: {}", e)));
                    status_msg.set("Error".to_string());
                    is_sending.set(false);
                    return;
                }
            };

            let client = RpcClient::new(transport, false);
            let provider = ProviderBuilder::new().connect_client(client);

            // Build transaction with from field (important!)
            let tx = TransactionRequest::default()
                .with_from(wallet_addr)
                .with_to(to_addr)
                .with_value(amount_wei);

            status_msg.set("Sending transaction (wallet will prompt)...".to_string());

            // Send transaction - WindowTransport routes to eth_sendTransaction
            // which prompts the browser wallet to sign and send
            match provider.send_transaction(tx).await {
                Ok(pending_tx) => {
                    let hash = *pending_tx.tx_hash();
                    tx_hash.set(Some(format!("{:?}", hash)));
                    status_msg.set("Transaction sent! Waiting for confirmation...".to_string());

                    tracing::info!("Transaction sent: {:?}", hash);

                    // Wait for confirmation
                    match pending_tx.get_receipt().await {
                        Ok(receipt) => {
                            tracing::info!(
                                "Transaction confirmed in block: {:?}",
                                receipt.block_number
                            );
                            status_msg.set("Transaction confirmed!".to_string());
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Confirmation error: {}", e)));
                            status_msg.set("Transaction sent but confirmation failed".to_string());
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to send transaction: {}", e)));
                    status_msg.set("Transaction failed".to_string());
                }
            }

            is_sending.set(false);
        });
    };

    rsx! {
        div { class: "h-full flex flex-col",
            // Header
            div { class: "mb-6",
                h2 { class: "text-2xl font-bold mb-2 bg-gradient-to-r from-green-400 to-emerald-500 bg-clip-text text-transparent",
                    "💸 Send Transaction"
                }
                p { class: "text-gray-400 text-sm", "Send ETH via eth_sendTransaction" }
            }

            // Status
            div { class: "mb-4 p-3 bg-blue-500/10 border border-blue-500/30 rounded-lg",
                p { class: "text-blue-300 text-sm flex items-center gap-2",
                    span { class: "text-base", "●" }
                    "{status_msg}"
                }
            }

            // Error message
            if let Some(err) = error_msg() {
                div { class: "mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg",
                    p { class: "text-red-300 text-sm", "⚠ {err}" }
                }
            }

            // Transaction hash
            if let Some(hash) = tx_hash() {
                div { class: "mb-4 p-3 bg-green-500/10 border border-green-500/30 rounded-lg",
                    p { class: "text-xs font-semibold text-green-400 mb-1", "Transaction Hash" }
                    code { class: "text-xs text-green-300 font-mono break-all block",
                        "{hash}"
                    }
                }
            }

            // Connect or show form
            if wallet_address().is_none() {
                div { class: "flex-1 flex items-center justify-center",
                    button {
                        class: "px-6 py-3 bg-gradient-to-r from-green-600 to-green-500 text-white rounded-lg hover:from-green-500 hover:to-green-400 transition-all duration-200 font-semibold shadow-lg shadow-green-500/50",
                        onclick: connect_wallet,
                        "Connect Wallet"
                    }
                }
            } else {
                div { class: "flex-1 flex flex-col gap-3",
                    // Recipient input
                    div {
                        label { class: "block text-xs font-semibold text-gray-400 mb-2",
                            "Recipient Address"
                        }
                        input {
                            class: "w-full px-3 py-2 text-sm text-white bg-gray-900/70 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-green-500 font-mono",
                            r#type: "text",
                            placeholder: "0x...",
                            value: "{recipient}",
                            oninput: move |evt| recipient.set(evt.value()),
                            disabled: is_sending(),
                        }
                    }

                    // Amount input
                    div {
                        label { class: "block text-xs font-semibold text-gray-400 mb-2",
                            "Amount (ETH)"
                        }
                        input {
                            class: "w-full px-3 py-2 text-sm text-white bg-gray-900/70 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-green-500",
                            r#type: "text",
                            placeholder: "0.001",
                            value: "{amount}",
                            oninput: move |evt| amount.set(evt.value()),
                            disabled: is_sending(),
                        }
                    }

                    // Send button
                    button {
                        class: "w-full px-4 py-3 bg-gradient-to-r from-green-600 to-green-500 text-white rounded-lg hover:from-green-500 hover:to-green-400 transition-all duration-200 font-semibold shadow-lg shadow-green-500/50 disabled:opacity-50 disabled:cursor-not-allowed",
                        onclick: send_tx,
                        disabled: is_sending() || recipient().is_empty(),
                        if is_sending() {
                            "Sending..."
                        } else {
                            "💸 Send Transaction"
                        }
                    }

                    // Info note
                    div { class: "mt-auto p-3 bg-blue-500/5 border border-blue-500/20 rounded-lg",
                        p { class: "text-xs text-blue-300 leading-relaxed",
                            "💡 Browser wallets use "
                            code { class: "text-blue-400 bg-gray-900/50 px-1.5 py-0.5 rounded font-mono text-xs",
                                "eth_sendTransaction"
                            }
                            " which signs AND broadcasts in one call. No wallet attachment needed - just send the transaction!"
                        }
                    }
                }
            }
        }
    }
}
