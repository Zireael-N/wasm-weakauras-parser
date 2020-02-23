mod decoder;
mod deserializer;
use decoder::decode_for_print;
use deserializer::deserialize;
pub use deserializer::LuaValue;

use inflate::inflate_bytes;

/// Takes a string encoded by WeakAuras and returns
/// a Vec of [LuaValues](enum.LuaValue.html).
pub fn decode(data: &str) -> Result<Vec<LuaValue>, &'static str> {
    if !data.starts_with('!') {
        return Err("legacy WAs are not supported");
    }

    let decoded = decode_for_print(&data[1..]);
    let decompressed = inflate_bytes(&decoded).map_err(|_| "failed to INFLATE")?;
    deserialize(std::str::from_utf8(&decompressed).map_err(|_| "invalid UTF-8")?)
}
