//! WindowTransport implementation - routes Alloy RPC calls through window.ethereum

use alloy_json_rpc::{RequestPacket, ResponsePacket};
use alloy_transport::{TransportError, TransportFut};
use serde_json::Value;
use std::task::{Context, Poll};
use tower::Service;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::error::{Result, WindowError};

/// Get window.ethereum object
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

/// Transport that uses window.ethereum (EIP-1193)
#[derive(Clone, Debug)]
pub struct WindowTransport {
    ethereum: JsValue,
}

impl WindowTransport {
    /// Create a new WindowTransport from window.ethereum
    pub fn new() -> Result<Self> {
        let ethereum = get_ethereum();

        if ethereum.is_null() || ethereum.is_undefined() {
            return Err(WindowError::NoWallet);
        }

        Ok(Self { ethereum })
    }

    /// Make a single RPC request
    async fn request_inner(&self, method: String, params: Value) -> Result<Value> {
        // For eth_call, transform "input" to "data" since window.ethereum expects "data"
        let params = if method == "eth_call" {
            tracing::debug!("Original params: {:?}", params);
            match params {
                Value::Array(mut arr) if arr.len() > 0 => {
                    // Transform the first element (the transaction object)
                    if let Some(Value::Object(obj)) = arr.get(0) {
                        if obj.contains_key("input") {
                            tracing::debug!("Found 'input', transforming to 'data'");
                            // Rebuild the object with "data" instead of "input"
                            let mut new_obj = serde_json::Map::new();
                            for (k, v) in obj {
                                if k == "input" {
                                    new_obj.insert("data".to_string(), v.clone());
                                } else {
                                    new_obj.insert(k.clone(), v.clone());
                                }
                            }
                            tracing::debug!("New object: {:?}", new_obj);
                            arr[0] = Value::Object(new_obj);
                        }
                    }
                    tracing::debug!("Transformed params: {:?}", arr);
                    Value::Array(arr)
                }
                _ => params,
            }
        } else {
            params
        };

        // Convert serde_json::Value to JsValue manually using js_sys
        // This avoids serde_wasm_bindgen serialization issues with Map types
        // MetaMask requires params to be an array or object, not null
        let params_js = match &params {
            Value::Null => {
                // Convert null to empty array for MetaMask compatibility
                let arr = js_sys::Array::new();
                arr.into()
            }
            _ => self.json_to_js(&params)?,
        };

        // Log the JS value
        let params_str = js_sys::JSON::stringify(&params_js)
            .map(|s| s.as_string().unwrap_or_default())
            .unwrap_or_default();
        tracing::debug!("params_js as JSON: {}", params_str);

        // Make the request
        let promise = ethereum_request(&self.ethereum, &method, &params_js);
        let result = JsFuture::from(promise).await?;

        tracing::debug!("Result: {:?}", result);

        // Convert back to serde_json::Value
        Ok(serde_wasm_bindgen::from_value(result)?)
    }

    /// Convert serde_json::Value to JsValue manually
    /// This is needed because serde_wasm_bindgen has issues with Map serialization
    fn json_to_js(&self, value: &Value) -> Result<JsValue> {
        match value {
            Value::Null => Ok(JsValue::NULL),
            Value::Bool(b) => Ok(JsValue::from(*b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(JsValue::from(i as f64))
                } else if let Some(u) = n.as_u64() {
                    Ok(JsValue::from(u as f64))
                } else if let Some(f) = n.as_f64() {
                    Ok(JsValue::from(f))
                } else {
                    Ok(JsValue::NULL)
                }
            }
            Value::String(s) => Ok(JsValue::from_str(s)),
            Value::Array(arr) => {
                let js_array = js_sys::Array::new();
                for item in arr {
                    js_array.push(&self.json_to_js(item)?);
                }
                Ok(js_array.into())
            }
            Value::Object(obj) => {
                let js_object = js_sys::Object::new();
                for (key, val) in obj {
                    let js_val = self.json_to_js(val)?;
                    js_sys::Reflect::set(&js_object, &JsValue::from_str(key), &js_val)
                        .map_err(|_| WindowError::SerializationError)?;
                }
                Ok(js_object.into())
            }
        }
    }
}

impl Service<RequestPacket> for WindowTransport {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        // Always ready since we're using window.ethereum
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let ethereum = self.ethereum.clone();
        let transport = Self { ethereum };

        Box::pin(async move {
            match req {
                RequestPacket::Single(single) => {
                    let method = single.method().to_string();

                    // Parse params from RawValue to Value
                    let params = match single.params() {
                        Some(raw) => serde_json::from_str(raw.get())
                            .map_err(|e| TransportError::local_usage(e))?,
                        None => Value::Null,
                    };

                    match transport.request_inner(method, params).await {
                        Ok(result) => {
                            // Build successful response
                            let response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": single.id(),
                                "result": result,
                            });
                            let response_packet = serde_json::from_value(response)
                                .map_err(|e| TransportError::local_usage(e))?;
                            Ok(ResponsePacket::Single(response_packet))
                        }
                        Err(e) => Err(TransportError::local_usage_str(&e.to_string())),
                    }
                }
                RequestPacket::Batch(batch) => {
                    // Process each request in the batch
                    let mut responses = Vec::new();

                    for single in batch.iter() {
                        let method = single.method().to_string();

                        // Parse params from RawValue to Value
                        let params = match single.params() {
                            Some(raw) => serde_json::from_str(raw.get())
                                .map_err(|e| TransportError::local_usage(e))?,
                            None => Value::Null,
                        };

                        match transport.request_inner(method, params).await {
                            Ok(result) => {
                                let response = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": single.id(),
                                    "result": result,
                                });
                                responses.push(response);
                            }
                            Err(e) => {
                                let error_response = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": single.id(),
                                    "error": {
                                        "code": -32000,
                                        "message": e.to_string(),
                                    }
                                });
                                responses.push(error_response);
                            }
                        }
                    }

                    let response_packet = serde_json::from_value(Value::Array(responses))
                        .map_err(|e| TransportError::local_usage(e))?;
                    Ok(ResponsePacket::Batch(response_packet))
                }
            }
        })
    }
}

// Transport trait is automatically implemented via the blanket impl
// when Service<RequestPacket> is implemented

// SAFETY: WASM is single-threaded, so Send and Sync are safe to implement
// even though JsValue is not Send/Sync. These traits are only used for
// multi-threaded environments, which don't exist in WASM.
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WindowTransport {}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WindowTransport {}
