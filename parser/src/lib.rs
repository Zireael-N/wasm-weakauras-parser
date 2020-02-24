mod decoder;
mod deserializer;
use decoder::{legacy, modern};
use deserializer::deserialize;
pub use deserializer::LuaValue;

/// Takes a string encoded by WeakAuras and returns
/// a Vec of [LuaValues](enum.LuaValue.html).
pub fn decode(data: &str) -> Result<Vec<LuaValue>, &'static str> {
    let decoded = if data.starts_with('!') {
        modern::decode(&data[1..])?
    } else {
        legacy::decode(data)?
    };

    deserialize(std::str::from_utf8(&decoded).map_err(|_| "invalid UTF-8")?)
}
