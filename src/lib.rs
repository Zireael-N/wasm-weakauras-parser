use js_sys::Error as JsError;
use std::error::Error;
use wasm_bindgen::prelude::*;
use weakauras_parser::decode;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(wa_string: &str) -> Result<String, JsValue> {
    let deserialized = decode(wa_string).map_err(|e| JsValue::from(JsError::new(e)))?;
    serde_json::to_string_pretty(&deserialized).map_err(|e| JsError::new(e.description()).into())
}
