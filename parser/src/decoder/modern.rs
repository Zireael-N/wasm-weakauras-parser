// This file is based on code from LibDeflate
// Copyright (C) 2018-2019 Haoqian He
// https://github.com/SafeteeWoW/LibDeflate

use super::byte_map::BYTE_MAP;
use inflate::inflate_bytes;

#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
static POW2: [u32; 31] = [
    1,        2,        4,        8,         16,        32,        64,         128,
    256,      512,      1024,     2048,      4096,      8192,      16384,      32768,
    65536,    131072,   262144,   524288,    1048576,   2097152,   4194304,    8388608,
    16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824,
];

pub(crate) fn decode_for_print(s: &str) -> Vec<u8> {
    let len = s.len();
    let mut result = Vec::with_capacity(len * 3 / 4);

    let mut chunks = s.as_bytes().chunks_exact(4);

    for chunk in chunks.by_ref() {
        let tripple = chunk
            .iter()
            .map(|&b| u32::from(BYTE_MAP[b]))
            .enumerate()
            .fold(0, |acc, (i, byte)| acc + (byte << (i * 6)));

        result.push(tripple as u8);
        result.push((tripple >> 8) as u8);
        result.push((tripple >> 16) as u8);
    }

    let mut cache = 0;
    let mut cache_bitlen = 0;
    for &byte in chunks.remainder() {
        cache += u32::from(BYTE_MAP[byte]) * POW2[cache_bitlen];
        cache_bitlen += 6;
    }
    while cache_bitlen >= 8 {
        result.push(cache as u8);
        cache >>= 8;
        cache_bitlen -= 8;
    }

    result
}

pub fn decode(data: &str) -> Result<Vec<u8>, &'static str> {
    let decoded = decode_for_print(data);
    inflate_bytes(&decoded).map_err(|_| "failed to INFLATE")
}
