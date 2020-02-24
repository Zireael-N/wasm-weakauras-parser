// This file is based on code from LibDeflate
// Copyright (C) 2018-2019 Haoqian He
// https://github.com/SafeteeWoW/LibDeflate

use core::ops::{Index, IndexMut};

struct ByteMap([u8; 256]);

impl ByteMap {
    #[inline(always)]
    fn get(&self, byte: u8) -> &u8 {
        // safety: this is safe because the underlying array has 256 elements in it
        unsafe { self.0.get_unchecked(byte as usize) }
    }

    #[inline(always)]
    fn get_mut(&mut self, byte: u8) -> &mut u8 {
        // safety: this is safe because the underlying array has 256 elements in it
        unsafe { self.0.get_unchecked_mut(byte as usize) }
    }
}

impl Index<u8> for ByteMap {
    type Output = u8;

    #[inline(always)]
    fn index(&self, byte: u8) -> &Self::Output {
        self.get(byte)
    }
}

impl IndexMut<u8> for ByteMap {
    #[inline(always)]
    fn index_mut(&mut self, byte: u8) -> &mut Self::Output {
        self.get_mut(byte)
    }
}

#[rustfmt::skip]
static BYTE_MAP: ByteMap = ByteMap([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    62, 63, // '(', ')'
    0, 0, 0, 0, 0, 0,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, // '0' - '9'
    0, 0, 0, 0, 0, 0, 0,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, // 'A' - 'Z'
    0, 0, 0, 0, 0, 0, 0,
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, // 'a' - 'z'
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

#[allow(clippy::unreadable_literal)]
#[rustfmt::skip]
static POW2: [u32; 31] = [
    1,        2,        4,        8,         16,        32,        64,         128,
    256,      512,      1024,     2048,      4096,      8192,      16384,      32768,
    65536,    131072,   262144,   524288,    1048576,   2097152,   4194304,    8388608,
    16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824,
];

// This is a faithful reimplementation of the Lua code,
// it's not idiomatic Rust.
pub(crate) fn decode_for_print(s: &str) -> Vec<u8> {
    let len = s.len();
    let mut result = Vec::with_capacity(len * 3 / 4);
    let mut iter = s.bytes().map(|b| u32::from(BYTE_MAP[b]));

    {
        let mut at = 0;
        loop {
            if at >= len - 4 {
                break;
            }

            let tripple = iter
                .by_ref()
                .take(4)
                .enumerate()
                .fold(0, |acc, (i, byte)| acc + (byte << (i * 6)));

            result.push(tripple as u8);
            result.push((tripple >> 8) as u8);
            result.push((tripple >> 16) as u8);

            at += 4;
        }
    }

    let mut cache = 0;
    let mut cache_bitlen = 0;
    for byte in iter {
        cache += byte * POW2[cache_bitlen];
        cache_bitlen += 6;
    }
    while cache_bitlen >= 8 {
        result.push(cache as u8);
        cache >>= 8;
        cache_bitlen -= 8;
    }

    result
}
