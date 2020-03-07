// This file is based on code from WeakAuras-Decoder
// Copyright (C) SoftCreatR
// https://github.com/SoftCreatR/WeakAuras-Decoder

use std::borrow::Cow;
use std::collections::BTreeMap as Map;

// This is far from idiomatic Rust.
pub(crate) fn decompress<'a>(bytes: &'a [u8]) -> Result<Cow<'a, [u8]>, &'static str> {
    let mut iter = bytes.iter();
    match iter.next() {
        Some(1) => return Ok(Cow::from(&bytes[1..])),
        Some(3) => (),
        _ => return Err("unknown compression codec"),
    }

    let len = bytes.len();
    if len < 5 {
        return Err("insufficient data");
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
            u32::from(*iter.next().ok_or("unexpected end of input")?) << bitfield_len;
        bitfield_len += 8;
        let symbol = bitfield as u8;
        bitfield >>= 8;
        bitfield_len -= 8;

        loop {
            bitfield +=
                u32::from(*iter.next().ok_or("unexpected end of input")?) << bitfield_len;
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
                if let Some(top_entry) = map.get(&l) {
                    let key = bitfield & ((1 << l) - 1);

                    if let Some(&inner_entry) = top_entry.get(&key) {
                        result.push(inner_entry);
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
