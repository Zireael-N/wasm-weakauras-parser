// This file is based on code from LibDeflate
// Copyright (C) 2018-2019 Haoqian He
// https://github.com/SafeteeWoW/LibDeflate

use super::byte_map::LOOKUP_TABLE;

pub(crate) fn decode(s: &str) -> Vec<u8> {
    let len = s.len();
    let mut result = Vec::with_capacity(len * 3 / 4);

    let mut chunks = s.as_bytes().chunks_exact(4);

    for chunk in chunks.by_ref() {
        let tripple = chunk
            .iter()
            .map(|&b| u32::from(LOOKUP_TABLE[b]))
            .enumerate()
            .fold(0, |acc, (i, byte)| acc + (byte << (i * 6)));

        result.push(tripple as u8);
        result.push((tripple >> 8) as u8);
        result.push((tripple >> 16) as u8);
    }

    let mut cache = 0;
    let mut cache_bitlen = 0;
    for &byte in chunks.remainder() {
        cache += u32::from(LOOKUP_TABLE[byte]) << cache_bitlen;
        cache_bitlen += 6;
    }
    while cache_bitlen >= 8 {
        result.push(cache as u8);
        cache >>= 8;
        cache_bitlen -= 8;
    }

    result
}
