//! Wallet Connection Example
//!
//! Demonstrates connecting to a browser wallet using WindowSigner and displaying:
//! - Wallet address
//! - Chain ID
//! - Account balance

use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy::signers::Signer;
use alloy_transport_window::{WindowSigner, WindowTransport};
use alloy_dyn_abi::eip712::TypedData;
use dioxus::prelude::*;
use serde::Serialize;

alloy_sol_types::sol! {
    #[derive(Serialize)]
    struct BingusBongus {
        address who;
    }
}

#[component]
pub fn WalletConnect() -> Element {
    let mut window_signer = use_signal(|| Option::<WindowSigner>::None);
    let mut wallet_address = use_signal(|| Option::<Address>::None);
    let mut chain_id = use_signal(|| Option::<u64>::None);
    let mut balance = use_signal(|| Option::<U256>::None);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::from("Not connected"));

    // Sign message
    let sign_message = move |_| {
        spawn(async move {
            status_msg.set("Attempting to sign messages...".to_string());
            error_msg.set(None);

            if let Some(signer) = window_signer() {
                if let Err(e) = signer.sign_message(b"bingus bongus").await {
                    error_msg.set(Some(format!("Sign message error: {e}")));
                } else {
                    status_msg.set("Signed bingus bongus".to_string());
                }

                let domain = alloy::sol_types::eip712_domain! {
                    name: "bingus bongus",
                    version: "0.1.0",
                };
                let data = BingusBongus {
                    who: [0xff; 20].into(),
                };
                let typed_data = TypedData::from_struct(&data, Some(domain));

                status_msg.set(format!("Attempting to sign: {typed_data:?}"));

                if let Err(e) = signer.sign_dynamic_typed_data(&typed_data).await {
                    error_msg.set(Some(format!("Sign EIP-712 error: {e}")));
                } else {
                    status_msg.set("Signed EIP-712 data".to_string());
                }
            }
        });
    };

    // Connect to wallet
    let connect_wallet = move |_| {
        spawn(async move {
            status_msg.set("Connecting to wallet...".to_string());
            error_msg.set(None);

            match WindowSigner::new().await {
                Ok(signer) => {
                    let addr = signer.address();
                    wallet_address.set(Some(addr));
                    window_signer.set(Some(signer));
                    status_msg.set("Connected!".to_string());

                    // Create provider and fetch basic data
                    match WindowTransport::new() {
                        Ok(transport) => {
                            let client = RpcClient::new(transport, false);
                            let provider = ProviderBuilder::new().connect_client(client);

                            // Fetch chain ID
                            if let Ok(id) = provider.get_chain_id().await {
                                chain_id.set(Some(id));
                            }

                            // Fetch balance
                            if let Ok(bal) = provider.get_balance(addr).await {
                                balance.set(Some(bal));
                            }
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Transport error: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to connect: {}", e)));
                    status_msg.set("Connection failed".to_string());
                }
            }
        });
    };

    rsx! {
        div { class: "h-full flex flex-col",
            // Header
            div { class: "mb-6",
                h2 { class: "text-2xl font-bold mb-2 bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent",
                    "🔗 Wallet Connection"
                }
                p { class: "text-gray-400 text-sm",
                    "Connect to MetaMask, Rabby, or any browser wallet"
                }
            }

            // Status message
            div { class: "mb-4 p-3 bg-blue-500/10 border border-blue-500/30 rounded-lg",
                p { class: "text-blue-300 text-sm flex items-center gap-2",
                    span { class: "text-base", "●" }
                    "{status_msg}"
                }
            }

            // Error message
            if let Some(err) = error_msg() {
                div { class: "mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg",
                    p { class: "text-red-300 text-sm flex items-center gap-2",
                        span { class: "text-base", "⚠" }
                        "Error: {err}"
                    }
                }
            }

            // Connect button or wallet info
            if wallet_address().is_none() {
                div { class: "flex-1 flex items-center justify-center",
                    button {
                        class: "px-6 py-3 bg-gradient-to-r from-blue-600 to-blue-500 text-white rounded-lg hover:from-blue-500 hover:to-blue-400 transition-all duration-200 font-semibold shadow-lg shadow-blue-500/50",
                        onclick: connect_wallet,
                        "Connect Wallet"
                    }
                }
            } else if let Some(addr) = wallet_address() {
                div { class: "flex-1 flex flex-col gap-3",
                    // Address
                    div { class: "p-4 bg-gray-900/50 rounded-lg border border-gray-700/50",
                        p { class: "text-xs font-semibold text-gray-400 mb-1", "Address" }
                        code { class: "text-xs text-blue-400 font-mono break-all block",
                            "{addr}"
                        }
                    }

                    // Chain ID
                    div { class: "p-4 bg-gray-900/50 rounded-lg border border-gray-700/50",
                        p { class: "text-xs font-semibold text-gray-400 mb-1", "Chain ID" }
                        if let Some(id) = chain_id() {
                            p { class: "text-xl font-bold text-purple-400",
                                "{id}"
                                if id == 42161 {
                                    span { class: "ml-2 text-xs text-green-400 bg-green-400/10 px-2 py-0.5 rounded-full",
                                        "Arbitrum"
                                    }
                                }
                            }
                        } else {
                            p { class: "text-sm text-gray-500", "Loading..." }
                        }
                    }

                    // Balance
                    if let Some(bal) = balance() {
                        div { class: "p-4 bg-gray-900/50 rounded-lg border border-gray-700/50",
                            p { class: "text-xs font-semibold text-gray-400 mb-1",
                                "Balance"
                            }
                            p { class: "text-lg font-bold text-green-400 font-mono break-all",
                                "{bal} wei"
                            }
                        }
                    }

                    div { class: "flex-1 flex items-center justify-center",
                        button {
                            class: "px-6 py-3 bg-gradient-to-r from-blue-600 to-blue-500 text-white rounded-lg hover:from-blue-500 hover:to-blue-400 transition-all duration-200 font-semibold shadow-lg shadow-blue-500/50",
                            onclick: sign_message,
                            "Sign Message"
                        }
                    }
                }
            }
        }
    }
}
