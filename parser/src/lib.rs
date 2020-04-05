mod base64;
mod deserializer;
mod huffman;
use deserializer::Deserializer;
pub use deserializer::LuaValue;

use std::borrow::Cow;

/// Takes a string encoded by WeakAuras and returns
/// a Vec of [LuaValues](enum.LuaValue.html).
pub fn decode(mut data: &str) -> Result<Vec<LuaValue>, &'static str> {
    let legacy = if data.starts_with('!') {
        data = &data[1..];
        false
    } else {
        true
    };

    let data = base64::decode(data)?;
    let decoded = if legacy {
        huffman::decompress(&data)?
    } else {
        Cow::from(inflate::inflate_bytes(&data).map_err(|_| "failed to INFLATE")?)
    };

    Deserializer::from_str(&String::from_utf8_lossy(&decoded)).deserialize()
}
