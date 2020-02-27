mod base64;
mod deserializer;
mod huffman;
use deserializer::deserialize;
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

    let data = base64::decode(data);
    let decoded = if legacy {
        huffman::decompress(&data)?
    } else {
        Cow::from(inflate::inflate_bytes(&data).map_err(|_| "failed to INFLATE")?)
    };

    deserialize(std::str::from_utf8(&decoded).map_err(|_| "invalid UTF-8")?)
}
