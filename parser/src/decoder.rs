struct ByteMap([u8; 256]);

impl ByteMap {
    #[inline(always)]
    fn get(&self, byte: u8) -> &u8 {
        // safety: this is safe because the underlying array has 256 elements in it
        unsafe { self.0.get_unchecked(byte as usize) }
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn get_mut(&mut self, byte: u8) -> &mut u8 {
        // safety: this is safe because the underlying array has 256 elements in it
        unsafe { self.0.get_unchecked_mut(byte as usize) }
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
    1,        2,        4,        8,         16,        32,        64,      128,
    256,      512,      1024,     2048,      4096,      8192,      16384,   32768,
    65536,    131072,   262144,   524288,    1048576,   2097152,   4194304, 8388608,
    16777216, 33554432, 67108864, 134217728, 268435456, 536870912, 1073741824,
];

// This is a faithful reimplementation of the Lua code,
// it's not idiomatic Rust.
pub(crate) fn decode_for_print(s: &str) -> Vec<u8> {
    let len = s.len();

    let mut result = Vec::with_capacity(len);

    let mut i = 0;
    loop {
        if i >= len - 4 {
            break;
        }

        let bytes: Vec<u8> = s[i..i + 4].bytes().map(|b| *BYTE_MAP.get(b)).collect();

        let mut cache = u32::from(bytes[0])
            + (u32::from(bytes[1]) << 6)
            + (u32::from(bytes[2]) << 12)
            + (u32::from(bytes[3]) << 18);
        let b1 = cache % 256;
        cache = (cache - b1) / 256;
        let b2 = cache % 256;
        let b3 = (cache - b2) / 256;
        result.push(b1 as u8);
        result.push(b2 as u8);
        result.push(b3 as u8);

        i += 4;
    }

    let mut cache = 0;
    let mut cache_bitlen = 0;
    loop {
        if i >= len - 1 {
            break;
        }

        let byte = *BYTE_MAP.get(s.as_bytes()[i]);
        cache += u32::from(byte) * POW2[cache_bitlen];

        i += 1;
    }

    while cache_bitlen >= 8 {
        let byte = cache % 256;
        result.push(byte as u8);
        cache = (cache - byte) / 256;
        cache_bitlen -= 8;
    }

    result
}
