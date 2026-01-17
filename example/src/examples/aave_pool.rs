//! Aave Pool Example
//!
//! Demonstrates using eth_call to fetch user account data from Aave V3 Pool contract.
//! Shows how to:
//! - Use the sol! macro to generate contract bindings
//! - Call view functions on smart contracts
//! - Parse and display structured contract data

use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::client::RpcClient;
use alloy::signers::Signer;
use alloy::sol;
use alloy_transport_window::{WindowSigner, WindowTransport};
use dioxus::logger::tracing;
use dioxus::prelude::*;

// Define the Aave Pool interface using sol! macro
sol! {
    #[sol(rpc)]
    IPool,
    "sol_interface/L2Pool.json"
}

#[component]
pub fn AavePool() -> Element {
    let mut wallet_address = use_signal(|| Option::<Address>::None);
    let mut pool_address =
        use_signal(|| String::from("0x794a61358d6845594f94dc1db02a252b5b4814ad")); // Aave V3 Pool on Arbitrum
    let mut account_data = use_signal(|| Option::<String>::None);
    let mut contract_info = use_signal(|| Option::<String>::None);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut status_msg = use_signal(|| String::from("Not connected"));

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

    // Fetch user account data from Aave pool
    let fetch_account_data = move |_| {
        spawn(async move {
            account_data.set(None);
            contract_info.set(None);
            error_msg.set(None);
            status_msg.set("Fetching account data...".to_string());

            // Get wallet address
            let addr = match wallet_address() {
                Some(a) => a,
                None => {
                    error_msg.set(Some("Please connect wallet first".to_string()));
                    status_msg.set("Error".to_string());
                    return;
                }
            };

            // Parse pool address
            let pool_addr = match pool_address().parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    error_msg.set(Some(format!("Invalid pool address: {}", e)));
                    status_msg.set("Error".to_string());
                    return;
                }
            };

            // Create provider
            let transport = match WindowTransport::new() {
                Ok(t) => t,
                Err(e) => {
                    error_msg.set(Some(format!("Transport error: {}", e)));
                    status_msg.set("Error".to_string());
                    return;
                }
            };

            let client = RpcClient::new(transport, false);
            let provider = ProviderBuilder::new().connect_client(client);

            // Verify the contract exists at this address
            let code_len = match provider.get_code_at(pool_addr).await {
                Ok(code) => {
                    if code.is_empty() {
                        error_msg.set(Some(format!("No contract found at address {}", pool_addr)));
                        status_msg.set("Invalid contract address".to_string());
                        contract_info.set(Some("❌ No contract at this address".to_string()));
                        return;
                    }
                    code.len()
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to check contract: {}", e)));
                    status_msg.set("Error".to_string());
                    return;
                }
            };

            contract_info.set(Some(format!("✓ Contract verified: {} bytes", code_len)));
            status_msg.set("Calling getUserAccountData...".to_string());

            // Create pool contract instance
            let pool = IPool::new(pool_addr, &provider);

            // Verify it's an Aave pool by calling ADDRESSES_PROVIDER
            match pool.ADDRESSES_PROVIDER().call().await {
                Ok(addr) => {
                    tracing::info!("Pool's Addresses Provider: {:?}", addr);
                }
                Err(e) => {
                    tracing::error!("Failed to get Addresses Provider: {}", e);
                }
            }

            tracing::info!("Calling getUserAccountData for address: {:?}", addr);

            // Call getUserAccountData
            match pool.getUserAccountData(addr).call().await {
                Ok(data) => {
                    let info = format!(
                        "Total Collateral: {} wei\nTotal Debt: {} wei\nAvailable Borrows: {} wei\nLiquidation Threshold: {}\nLTV: {}\nHealth Factor: {}",
                        data.totalCollateralBase,
                        data.totalDebtBase,
                        data.availableBorrowsBase,
                        data.currentLiquidationThreshold,
                        data.ltv,
                        data.healthFactor
                    );
                    account_data.set(Some(info));
                    status_msg.set("Account data fetched!".to_string());
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to fetch account data: {}", e)));
                    status_msg.set("Fetch failed".to_string());
                }
            }
        });
    };

    rsx! {
        div { class: "h-full flex flex-col",
            // Header
            div { class: "mb-6",
                h2 { class: "text-2xl font-bold mb-2 bg-gradient-to-r from-purple-400 to-pink-500 bg-clip-text text-transparent",
                    "🏦 Aave Pool Data"
                }
                p { class: "text-gray-400 text-sm",
                    "Query Aave V3 using eth_call with type-safe bindings"
                }
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

            // Connect or show form
            if wallet_address().is_none() {
                div { class: "flex-1 flex items-center justify-center",
                    button {
                        class: "px-6 py-3 bg-gradient-to-r from-purple-600 to-purple-500 text-white rounded-lg hover:from-purple-500 hover:to-purple-400 transition-all duration-200 font-semibold shadow-lg shadow-purple-500/50",
                        onclick: connect_wallet,
                        "Connect Wallet"
                    }
                }
            } else {
                div { class: "flex-1 flex flex-col gap-3",
                    // Pool address input
                    div {
                        label { class: "block text-xs font-semibold text-gray-400 mb-2",
                            "Pool Address"
                        }
                        input {
                            class: "w-full px-3 py-2 text-sm text-white bg-gray-900/70 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 font-mono",
                            r#type: "text",
                            placeholder: "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2",
                            value: "{pool_address}",
                            oninput: move |evt| pool_address.set(evt.value()),
                        }
                        p { class: "text-xs text-gray-500 mt-1", "Default: Aave V3 Pool on Arbitrum" }
                    }

                    // Contract verification
                    if let Some(info) = contract_info() {
                        div { class: "p-3 bg-gray-900/50 border border-gray-700/50 rounded-lg",
                            p { class: "text-xs text-gray-300 font-mono", "{info}" }
                        }
                    }

                    // Fetch button
                    button {
                        class: "w-full px-4 py-3 bg-gradient-to-r from-purple-600 to-purple-500 text-white rounded-lg hover:from-purple-500 hover:to-purple-400 transition-all duration-200 font-semibold shadow-lg shadow-purple-500/50",
                        onclick: fetch_account_data,
                        "📊 Fetch Account Data"
                    }

                    // Account data display
                    if let Some(data) = account_data() {
                        div { class: "flex-1 p-4 bg-purple-500/10 border border-purple-500/30 rounded-lg overflow-auto",
                            p { class: "text-xs font-semibold text-purple-400 mb-2",
                                "✓ Account Data Retrieved"
                            }
                            pre { class: "text-xs text-purple-200 font-mono whitespace-pre-wrap",
                                "{data}"
                            }
                        }
                    }
                }
            }
        }
    }
}
