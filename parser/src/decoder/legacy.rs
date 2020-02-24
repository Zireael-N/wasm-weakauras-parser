// This file is based on code from
// 1) LibCompress (decode_for_print function)
//    Copyright (C) jjsheets and Galmok
//    https://www.curseforge.com/wow/addons/libcompress
// 2) WeakAuras-Decoder (decompress_huffman function)
//    Copyright (C) SoftCreatR
//    https://github.com/SoftCreatR/WeakAuras-Decoder

use super::byte_map::BYTE_MAP;
use std::borrow::Cow;
use std::collections::{btree_map::Entry, BTreeMap as Map};

pub(crate) fn decode_for_print(s: &str) -> Vec<u8> {
    let len = s.len();
    let mut result = Vec::with_capacity(len * 3 / 4);

    let mut bitfield = 0;
    let mut bitfield_len = 0;
    for byte in s.bytes().map(|b| u32::from(BYTE_MAP[b])) {
        if bitfield_len >= 8 {
            result.push(bitfield as u8);
            bitfield >>= 8;
            bitfield_len -= 8;
        }
        bitfield += byte << bitfield_len;
        bitfield_len += 6;
    }

    result
}

// This is far from idiomatic Rust.
pub(crate) fn decompress_huffman<'a>(bytes: &'a [u8]) -> Result<Cow<'a, [u8]>, &'static str> {
    let len = bytes.len();
    if len < 5 {
        return Err("insufficient data");
    }

    let mut iter = bytes.iter();
    match iter.next() {
        Some(1) => return Ok(Cow::from(&bytes[1..])),
        Some(3) => (),
        _ => return Err("unknown compression codec"),
    }

    let num_symbols = iter.next().unwrap() + 1;
    let original_size = iter
        .by_ref()
        .take(3)
        .map(|&byte| usize::from(byte))
        .enumerate()
        .fold(0, |acc, (i, byte)| acc + (byte << (i * 8)));

    if original_size == 0 {
        return Err("insufficient data");
    }

    let mut map: Map<u32, Map<u32, u8>> = Map::new();
    let mut result = Vec::with_capacity(original_size);

    let mut bitfield = 0u32;
    let mut bitfield_len = 0u32;

    for _ in 0..num_symbols {
        bitfield +=
            u32::from(*iter.next().ok_or_else(|| "unexpected end of input")?) << bitfield_len;
        bitfield_len += 8;
        let symbol = bitfield as u8;
        bitfield >>= 8;
        bitfield_len -= 8;

        loop {
            bitfield +=
                u32::from(*iter.next().ok_or_else(|| "unexpected end of input")?) << bitfield_len;
            bitfield_len += 8;
            if (bitfield & (bitfield >> 1)) != 0 {
                break;
            }
        }

        let mut cut = 0u32;
        let mut l = 0u32;
        let mut code = 0u32;
        while ((bitfield >> cut) & 3) < 3 {
            if ((bitfield >> cut) & 1) != 0 {
                code += 1 << l;
                cut += 1;
            }
            l += 1;
            cut += 1;
        }

        let top_entry = map.entry(l).or_insert_with(Map::new);
        top_entry.insert(code, symbol);
        bitfield >>= cut + 2;
        bitfield_len -= cut + 2;
    }

    while bitfield_len <= 32 {
        loop {
            let mut l = 0;
            #[allow(clippy::mut_range_bound)]
            for i in bitfield_len.saturating_sub(7)..=bitfield_len {
                l = i;
                if let Entry::Occupied(mut top_entry) = map.entry(l) {
                    let key = bitfield & ((1 << l) - 1);

                    if let Entry::Occupied(inner_entry) = top_entry.get_mut().entry(key) {
                        result.push(*inner_entry.get());
                        bitfield >>= l;
                        bitfield_len -= l;
                        l = 0;

                        break;
                    }
                }
            }
            if l >= bitfield_len {
                break;
            }
        }

        if let Some(&byte) = iter.next() {
            bitfield += u32::from(byte) << bitfield_len;
            bitfield_len += 8;
        } else {
            break;
        }
    }

    if bitfield_len > 32 {
        Err("encoding is too long")
    } else {
        Ok(Cow::from(result))
    }
}

pub fn decode(data: &str) -> Result<Vec<u8>, &'static str> {
    let decoded = decode_for_print(data);
    Ok(decompress_huffman(&decoded)?.into_owned())
}
