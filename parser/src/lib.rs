#[cfg(not(fuzzing))]
mod base64;
#[cfg(fuzzing)]
pub mod base64;
#[cfg(not(fuzzing))]
mod huffman;
#[cfg(fuzzing)]
pub mod huffman;

#[cfg(not(fuzzing))]
mod deserialization;
#[cfg(fuzzing)]
pub mod deserialization;
#[cfg(not(fuzzing))]
mod serialization;
#[cfg(fuzzing)]
pub mod serialization;
mod value;

use deserialization::Deserializer;
use serialization::Serializer;
pub use value::LuaValue;

use std::borrow::Cow;

const MAX_SIZE: usize = 16 * 1024 * 1024;

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
        huffman::decompress(&data)
    } else {
        use flate2::read::DeflateDecoder;
        use std::io::prelude::*;

        let mut result = Vec::new();
        let mut inflater = DeflateDecoder::new(&data[..]).take(MAX_SIZE as u64);

        inflater
            .read_to_end(&mut result)
            .map_err(|_| "failed to INFLATE")
            .and_then(|_| {
                if result.len() < MAX_SIZE {
                    Ok(())
                } else {
                    match inflater.into_inner().bytes().next() {
                        Some(_) => Err("compressed data is too large"),
                        None => Ok(()),
                    }
                }
            })
            .map(|_| Cow::from(result))
    }?;

    Deserializer::from_str(&String::from_utf8_lossy(&decoded)).deserialize()
}

/// Takes a [LuaValue](enum.LuaValue.html) and returns
/// a string that can be decoded by WeakAuras.
pub fn encode(value: &LuaValue) -> Result<String, &'static str> {
    Serializer::serialize(value, None)
        .and_then(|serialized| {
            use flate2::{read::DeflateEncoder, Compression};
            use std::io::prelude::*;

            let mut result = Vec::new();
            let mut deflater = DeflateEncoder::new(serialized.as_bytes(), Compression::best());

            deflater
                .read_to_end(&mut result)
                .map(|_| result)
                .map_err(|_| "failed to DEFLATE")
        })
        .and_then(|compressed| base64::encode_weakaura(&compressed))
}
