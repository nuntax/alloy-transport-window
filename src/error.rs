//! Error types for window.ethereum interactions

use thiserror::Error;

/// Errors that can occur when interacting with window.ethereum
#[derive(Error, Debug)]
pub enum WindowError {
    /// window.ethereum is not available (no wallet installed)
    #[error("window.ethereum not found - no Web3 wallet installed")]
    NoWallet,

    /// User rejected the request in their wallet
    #[error("User rejected the request")]
    UserRejected,

    /// RPC error from the wallet
    #[error("RPC error: {0}")]
    Rpc(String),

    /// JavaScript interop error
    #[error("JS error: {0}")]
    Js(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_wasm_bindgen::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic serialization error
    #[error("Serialization failed")]
    SerializationError,

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid signature format
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// No accounts returned from wallet
    #[error("No accounts available")]
    NoAccounts,
}

impl From<wasm_bindgen::JsValue> for WindowError {
    fn from(val: wasm_bindgen::JsValue) -> Self {
        // Try to extract meaningful error message
        if let Some(s) = val.as_string() {
            // Check for user rejection
            if s.contains("User denied") || s.contains("rejected") || s.contains("User rejected") {
                return WindowError::UserRejected;
            }
            return WindowError::Js(s);
        }

        // Fallback to debug representation
        WindowError::Js(format!("{:?}", val))
    }
}

/// Result type alias for window.ethereum operations
pub type Result<T> = std::result::Result<T, WindowError>;
