use js_sys::Error as JsError;
use wasm_bindgen::prelude::*;
use weakauras_parser as parser;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn decode(wa_string: &str) -> Result<String, JsValue> {
    let deserialized = parser::decode(wa_string).map_err(|e| JsValue::from(JsError::new(e)))?;

    (if deserialized.len() == 1 {
        serde_json::to_string_pretty(&deserialized[0])
    } else {
        serde_json::to_string_pretty(&deserialized)
    })
    .map_err(|e| JsError::new(&e.to_string()).into())
}

#[wasm_bindgen]
pub fn encode(json: &str) -> Result<String, JsValue> {
    serde_json::from_str(json)
        .map_err(|e| JsValue::from(JsError::new(&e.to_string())))
        .and_then(|value| parser::encode(&value).map_err(|e| JsError::new(e).into()))
}
