use wasm_bindgen::prelude::*;
use weakauras_parser::decode;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(wa_string: &str) -> Option<String> {
    let deserialized = decode(wa_string).ok()?;
    serde_json::to_string_pretty(&deserialized).ok()
}
