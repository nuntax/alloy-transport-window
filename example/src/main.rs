use dioxus::prelude::*;

mod examples;
use examples::{AavePool, BlockExplorer, SendTransaction, WalletConnect};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Home page with 2x2 grid of examples
#[component]
fn Home() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900",
            div { class: "container mx-auto px-4 py-8 max-w-7xl",
                // Header
                div { class: "mb-8 text-center",
                    h1 { class: "text-4xl font-bold mb-3 bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent",
                        "Alloy + Dioxus Examples"
                    }
                    p { class: "text-gray-400 text-base",
                        "Rust-native Ethereum integration with "
                        code { class: "text-blue-400 bg-gray-800/50 px-2 py-1 rounded font-mono text-sm",
                            "alloy-transport-window"
                        }
                    }
                }

                // 2x2 Grid of Examples
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    // Example 1: Wallet Connect
                    ExampleCard {
                        title: "Wallet Connection",
                        icon: "🔗",
                        gradient: "from-blue-500/20 to-blue-600/20",
                        border: "border-blue-500/30",
                        WalletConnect {}
                    }

                    // Example 2: Aave Pool
                    ExampleCard {
                        title: "Aave Pool Data",
                        icon: "🏦",
                        gradient: "from-purple-500/20 to-purple-600/20",
                        border: "border-purple-500/30",
                        AavePool {}
                    }

                    // Example 3: Send Transaction
                    ExampleCard {
                        title: "Send Transaction",
                        icon: "💸",
                        gradient: "from-green-500/20 to-green-600/20",
                        border: "border-green-500/30",
                        SendTransaction {}
                    }

                    // Example 4: Block Explorer
                    ExampleCard {
                        title: "Block Explorer",
                        icon: "🔍",
                        gradient: "from-orange-500/20 to-orange-600/20",
                        border: "border-orange-500/30",
                        BlockExplorer {}
                    }
                }

                // Footer
                div { class: "mt-12 p-6 bg-gray-800/30 border border-gray-700/50 rounded-2xl backdrop-blur-sm",
                    p { class: "text-center text-gray-400 text-sm",
                        "Built with "
                        span { class: "text-blue-400 font-semibold", "Dioxus 0.7" }
                        " • "
                        span { class: "text-purple-400 font-semibold", "Alloy" }
                        " • "
                        span { class: "text-green-400 font-semibold", "Rust" }
                    }
                }
            }
        }
    }
}

/// Reusable card component for examples
#[component]
fn ExampleCard(title: String, icon: String, gradient: String, border: String, children: Element) -> Element {
    rsx! {
        div {
            class: "bg-gradient-to-br {gradient} border {border} rounded-2xl backdrop-blur-sm shadow-xl overflow-hidden",
            // Card header
            div { class: "p-4 border-b border-gray-700/50 bg-gray-900/30",
                h3 { class: "text-lg font-bold text-white flex items-center gap-2",
                    span { "{icon}" }
                    "{title}"
                }
            }
            // Card content
            div { class: "p-6 h-[500px] overflow-auto",
                {children}
            }
        }
    }
}

/// Shared navbar component
#[component]
fn Navbar() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}
