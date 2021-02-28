#[cfg(not(fuzzing))]
mod ace_serialize;
#[cfg(fuzzing)]
pub mod ace_serialize;

#[cfg(not(fuzzing))]
mod lib_serialize;
#[cfg(fuzzing)]
pub mod lib_serialize;

use ace_serialize::Deserializer as LegacyDeserializer;
use lib_serialize::{Deserializer, Serializer};
use lua_value::LuaValue;

use std::borrow::Cow;

const MAX_SIZE: usize = 16 * 1024 * 1024;

enum StringVersion {
    Huffman,             // base64
    Deflate,             // ! + base64
    BinarySerialization, // !WA:\d+! + base64
}

/// Takes a string encoded by WeakAuras and returns
/// a Vec of [LuaValues](../weakauras_parser_lua_value/enum.LuaValue.html).
pub fn decode(mut data: &str) -> Result<Vec<LuaValue>, &'static str> {
    let version = if data.starts_with("!WA:2!") {
        data = &data[6..];
        StringVersion::BinarySerialization
    } else if data.starts_with('!') {
        data = &data[1..];
        StringVersion::Deflate
    } else {
        StringVersion::Huffman
    };

    let data = wa_base64::decode(data)?;
    let decoded = if let StringVersion::Huffman = version {
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

    if let StringVersion::BinarySerialization = version {
        Deserializer::from_slice(&decoded).deserialize()
    } else {
        LegacyDeserializer::from_str(&String::from_utf8_lossy(&decoded)).deserialize()
    }
}

/// Takes a [LuaValue](../weakauras_parser_lua_value/enum.LuaValue.html) and returns
/// a string that can be decoded by WeakAuras.
pub fn encode(value: &LuaValue) -> Result<String, &'static str> {
    Serializer::serialize(value, None)
        .and_then(|serialized| {
            use flate2::{read::DeflateEncoder, Compression};
            use std::io::prelude::*;

            let mut result = Vec::new();
            let mut deflater = DeflateEncoder::new(serialized.as_slice(), Compression::best());

            deflater
                .read_to_end(&mut result)
                .map(|_| result)
                .map_err(|_| "failed to DEFLATE")
        })
        .and_then(|compressed| wa_base64::encode_with_prefix(&compressed, "!WA:2!"))
}
