//! Block Explorer Example
//!
//! Demonstrates querying blockchain data without requiring wallet connection.
//! Shows how to:
//! - Fetch latest block information
//! - Display block details
//! - Query blockchain state

use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy_transport_window::WindowTransport;
use dioxus::prelude::*;

#[component]
pub fn BlockExplorer() -> Element {
    let mut block_number = use_signal(|| Option::<u64>::None);
    let mut block_hash = use_signal(|| Option::<String>::None);
    let mut block_timestamp = use_signal(|| Option::<u64>::None);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::from("Ready"));
    let mut is_loading = use_signal(|| false);

    // Fetch latest block
    let fetch_block = move |_| {
        spawn(async move {
            is_loading.set(true);
            error_msg.set(None);
            status_msg.set("Fetching latest block...".to_string());

            // Create provider (no wallet needed for read operations)
            let transport = match WindowTransport::new() {
                Ok(t) => t,
                Err(e) => {
                    error_msg.set(Some(format!("Transport error: {}", e)));
                    status_msg.set("Error".to_string());
                    is_loading.set(false);
                    return;
                }
            };

            let client = RpcClient::new(transport, false);
            let provider = ProviderBuilder::new().connect_client(client);

            // Fetch latest block number
            match provider.get_block_number().await {
                Ok(num) => {
                    block_number.set(Some(num));

                    // Fetch block details
                    match provider.get_block_by_number(num.into()).await {
                        Ok(Some(block)) => {
                            block_hash.set(Some(format!("{:?}", block.header.hash)));
                            block_timestamp.set(Some(block.header.timestamp));
                            status_msg.set("Block fetched!".to_string());
                        }
                        Ok(None) => {
                            error_msg.set(Some("Block not found".to_string()));
                            status_msg.set("Error".to_string());
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Failed to fetch block: {}", e)));
                            status_msg.set("Error".to_string());
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to fetch block number: {}", e)));
                    status_msg.set("Error".to_string());
                }
            }

            is_loading.set(false);
        });
    };

    rsx! {
        div { class: "h-full flex flex-col",
            // Header
            div { class: "mb-6",
                h2 { class: "text-2xl font-bold mb-2 bg-gradient-to-r from-orange-400 to-red-500 bg-clip-text text-transparent",
                    "🔍 Block Explorer"
                }
                p { class: "text-gray-400 text-sm", "Query blockchain data without wallet connection" }
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

            // Fetch button
            div { class: "mb-4",
                button {
                    class: "w-full px-4 py-3 bg-gradient-to-r from-orange-600 to-orange-500 text-white rounded-lg hover:from-orange-500 hover:to-orange-400 transition-all duration-200 font-semibold shadow-lg shadow-orange-500/50 disabled:opacity-50 disabled:cursor-not-allowed",
                    onclick: fetch_block,
                    disabled: is_loading(),
                    if is_loading() {
                        "Fetching..."
                    } else {
                        "🔍 Fetch Latest Block"
                    }
                }
            }

            // Block info display
            if block_number().is_some() {
                div { class: "flex-1 flex flex-col gap-3",
                    // Block number
                    div { class: "p-4 bg-gray-900/50 rounded-lg border border-gray-700/50",
                        p { class: "text-xs font-semibold text-gray-400 mb-1", "Block Number" }
                        p { class: "text-xl font-bold text-orange-400", "{block_number().unwrap()}" }
                    }

                    // Block hash
                    if let Some(hash) = block_hash() {
                        div { class: "p-4 bg-gray-900/50 rounded-lg border border-gray-700/50",
                            p { class: "text-xs font-semibold text-gray-400 mb-1",
                                "Block Hash"
                            }
                            code { class: "text-xs text-blue-400 font-mono break-all block",
                                "{hash}"
                            }
                        }
                    }

                    // Block timestamp
                    if let Some(timestamp) = block_timestamp() {
                        div { class: "p-4 bg-gray-900/50 rounded-lg border border-gray-700/50",
                            p { class: "text-xs font-semibold text-gray-400 mb-1",
                                "Timestamp"
                            }
                            p { class: "text-lg font-bold text-green-400", "{timestamp}" }
                        }
                    }
                }
            }
        }
    }
}
