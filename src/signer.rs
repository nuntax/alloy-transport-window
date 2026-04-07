//! WindowSigner implementation - delegates signing to browser wallet

use alloy_primitives::{Address, Signature, B256};
use alloy_signer::{Result as SignerResult, Signer, UnsupportedSignerOperation};
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[cfg(feature = "eip712")]
use alloy_dyn_abi::eip712::TypedData;
#[cfg(feature = "eip712")]
use alloy_dyn_abi::Eip712Domain;
#[cfg(feature = "eip712")]
use alloy_sol_types::SolStruct;

use crate::error::{Result, WindowError};

/// Get window.ethereum object and make requests
#[wasm_bindgen(inline_js = r#"
export function get_ethereum() {
    if (typeof window !== 'undefined' && window.ethereum) {
        return window.ethereum;
    }
    return null;
}

export function ethereum_request(ethereum, method, params) {
    return ethereum.request({ method, params });
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = get_ethereum)]
    fn get_ethereum() -> JsValue;

    #[wasm_bindgen(js_name = ethereum_request)]
    fn ethereum_request(ethereum: &JsValue, method: &str, params: &JsValue) -> js_sys::Promise;
}

/// Signer that delegates to window.ethereum (EIP-1193)
#[derive(Clone, Debug)]
pub struct WindowSigner {
    ethereum: JsValue,
    address: Address,
    chain_id: Option<u64>,
}

impl WindowSigner {
    /// Create a new WindowSigner and request account access
    pub async fn new() -> Result<Self> {
        let ethereum = get_ethereum();

        if ethereum.is_null() || ethereum.is_undefined() {
            return Err(WindowError::NoWallet);
        }

        // Request accounts (will trigger wallet popup)
        let params = serde_wasm_bindgen::to_value(&json!([]))?;
        let promise = ethereum_request(&ethereum, "eth_requestAccounts", &params);
        let result = JsFuture::from(promise).await?;
        let accounts: Vec<String> = serde_wasm_bindgen::from_value(result)?;

        let address = accounts
            .first()
            .ok_or(WindowError::NoAccounts)?
            .parse()
            .map_err(|e| WindowError::InvalidAddress(format!("{}", e)))?;

        // Get chain ID
        let chain_params = serde_wasm_bindgen::to_value(&json!([]))?;
        let chain_promise = ethereum_request(&ethereum, "eth_chainId", &chain_params);
        let chain_result = JsFuture::from(chain_promise).await?;
        let chain_id_hex: String = serde_wasm_bindgen::from_value(chain_result)?;

        let chain_id = u64::from_str_radix(chain_id_hex.trim_start_matches("0x"), 16).ok();

        Ok(Self {
            ethereum,
            address,
            chain_id,
        })
    }

    /// Get the connected address without requesting permissions again
    pub async fn from_existing() -> Result<Self> {
        let ethereum = get_ethereum();

        if ethereum.is_null() || ethereum.is_undefined() {
            return Err(WindowError::NoWallet);
        }

        // Get accounts (doesn't prompt)
        let params = serde_wasm_bindgen::to_value(&json!([]))?;
        let promise = ethereum_request(&ethereum, "eth_accounts", &params);
        let result = JsFuture::from(promise).await?;
        let accounts: Vec<String> = serde_wasm_bindgen::from_value(result)?;

        let address = accounts
            .first()
            .ok_or(WindowError::NoAccounts)?
            .parse()
            .map_err(|e| WindowError::InvalidAddress(format!("{}", e)))?;

        // Get chain ID
        let chain_params = serde_wasm_bindgen::to_value(&json!([]))?;
        let chain_promise = ethereum_request(&ethereum, "eth_chainId", &chain_params);
        let chain_result = JsFuture::from(chain_promise).await?;
        let chain_id_hex: String = serde_wasm_bindgen::from_value(chain_result)?;

        let chain_id = u64::from_str_radix(chain_id_hex.trim_start_matches("0x"), 16).ok();

        Ok(Self {
            ethereum,
            address,
            chain_id,
        })
    }

    /// Sign statically-typed EIP-712 data by converting it to [`TypedData`] and
    /// delegating to `eth_signTypedData_v4`.
    ///
    /// This is the preferred way to sign `sol!`-generated structs. It requires
    /// `T: serde::Serialize` (in addition to [`SolStruct`]) so that the message
    /// can be serialised to the JSON object expected by the wallet.
    ///
    /// # Note
    /// The [`Signer::sign_typed_data`] trait method cannot be overridden with
    /// this implementation because Rust's trait coherence rules (E0276) forbid
    /// adding the extra `T: Serialize` bound in an `impl` block. Call this
    /// method directly instead.
    #[cfg(feature = "eip712")]
    pub async fn sign_eip712<T: SolStruct + serde::Serialize>(
        &self,
        payload: &T,
        domain: Eip712Domain,
    ) -> SignerResult<Signature> {
        let typed_data = TypedData::from_struct(payload, Some(domain));
        self.sign_dynamic_typed_data_impl(&typed_data).await
    }

    /// Helper method to sign EIP-712 typed data
    #[cfg(feature = "eip712")]
    async fn sign_dynamic_typed_data_impl(
        &self,
        typed_data: &TypedData,
    ) -> SignerResult<Signature> {
        let typed_data_value = serde_wasm_bindgen::to_value(typed_data).map_err(|e| {
            alloy_signer::Error::other(format!("Failed to serialize typed data: {}", e))
        })?;

        // Create params array: [address, typedData]
        let params_array = js_sys::Array::new();
        params_array.push(&JsValue::from_str(&self.address.to_string()));
        params_array.push(&typed_data_value);

        let params: JsValue = params_array.into();

        let promise = ethereum_request(&self.ethereum, "eth_signTypedData_v4", &params);
        let result = JsFuture::from(promise)
            .await
            .map_err(|e| alloy_signer::Error::other(WindowError::from(e).to_string()))?;

        let sig_hex: String = serde_wasm_bindgen::from_value(result)
            .map_err(|e| alloy_signer::Error::other(e.to_string()))?;

        sig_hex
            .parse()
            .map_err(|e| alloy_signer::Error::other(format!("Invalid signature: {}", e)))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Signer for WindowSigner {
    async fn sign_hash(&self, _hash: &B256) -> SignerResult<Signature> {
        Err(alloy_signer::Error::UnsupportedOperation(
            UnsupportedSignerOperation::SignHash,
        ))
    }

    async fn sign_message(&self, message: &[u8]) -> SignerResult<Signature> {
        let params = serde_wasm_bindgen::to_value(&json!([
            format!("0x{}", hex::encode(message)),
            self.address.to_string(),
        ]))
        .map_err(|e| alloy_signer::Error::other(e.to_string()))?;

        let promise = ethereum_request(&self.ethereum, "personal_sign", &params);
        let result = JsFuture::from(promise)
            .await
            .map_err(|e| alloy_signer::Error::other(WindowError::from(e).to_string()))?;

        let sig_hex: String = serde_wasm_bindgen::from_value(result)
            .map_err(|e| alloy_signer::Error::other(e.to_string()))?;

        sig_hex
            .parse()
            .map_err(|e| alloy_signer::Error::other(format!("Invalid signature: {}", e)))
    }

    #[cfg(feature = "eip712")]
    async fn sign_typed_data<T: SolStruct + Send + Sync>(
        &self,
        _payload: &T,
        _domain: &Eip712Domain,
    ) -> SignerResult<Signature> {
        // `eth_signTypedData_v4` requires the message as a JSON object, which
        // requires `T: serde::Serialize`. Rust's trait coherence rules forbid
        // adding that bound here (E0276). Use `sign_eip712` instead, which
        // accepts `T: SolStruct + Serialize` as an inherent method.
        Err(alloy_signer::Error::UnsupportedOperation(
            UnsupportedSignerOperation::SignTypedData,
        ))
    }

    #[cfg(feature = "eip712")]
    async fn sign_dynamic_typed_data(&self, payload: &TypedData) -> SignerResult<Signature> {
        self.sign_dynamic_typed_data_impl(payload).await
    }

    fn address(&self) -> Address {
        self.address
    }

    fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    fn set_chain_id(&mut self, chain_id: Option<u64>) {
        self.chain_id = chain_id;
    }
}

// SAFETY: WASM is single-threaded, so Send and Sync are safe to implement
// even though JsValue is not Send/Sync. These traits are only used for
// multi-threaded environments, which don't exist in WASM.
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WindowSigner {}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WindowSigner {}
