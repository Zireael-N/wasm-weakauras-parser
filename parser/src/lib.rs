mod base64;
mod huffman;

mod deserialization;
mod serialization;
mod value;

use deserialization::Deserializer;
use serialization::Serializer;
pub use value::LuaValue;

use deflate::{self, Compression};
use inflate;
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

/// Takes a [LuaValue](enum.LuaValue.html) and returns
/// a string that can be decoded by WeakAuras.
pub fn encode(value: &LuaValue) -> Result<String, &'static str> {
    Serializer::serialize(value, None)
        .map(|serialized| deflate::deflate_bytes_conf(serialized.as_bytes(), Compression::Best))
        .and_then(|compressed| base64::encode_weakaura(&compressed))
}
