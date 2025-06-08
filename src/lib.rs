use js_sys::Error as JsError;
use wasm_bindgen::prelude::*;
use weakauras_codec::{self, OutputStringVersion};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn decode(wa_string: &str) -> Result<String, JsValue> {
    let deserialized = weakauras_codec::decode(wa_string.as_bytes().trim_ascii_end(), None)
        .map_err(|e| JsValue::from(JsError::new(&e.to_string())))?;

    serde_json::to_string_pretty(&deserialized).map_err(|e| JsError::new(&e.to_string()).into())
}

#[wasm_bindgen]
pub fn encode(json: &str) -> Result<String, JsValue> {
    serde_json::from_str(json)
        .map_err(|e| JsValue::from(JsError::new(&e.to_string())))
        .and_then(|value| {
            weakauras_codec::encode(&value, OutputStringVersion::BinarySerialization)
                .map_err(|e| JsError::new(&e.to_string()).into())
        })
}
