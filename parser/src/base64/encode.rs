use super::byte_map::ENCODE_LUT;

#[cfg(not(feature = "unsafe"))]
pub(crate) fn encode(data: &[u8]) -> String {
    let mut result = Vec::with_capacity((data.len() * 4 + 2) / 3);
    let mut chunks = data.chunks_exact(3);

    for chunk in chunks.by_ref() {
        let word = u32::from(chunk[0]) + (u32::from(chunk[1]) << 8) + (u32::from(chunk[2]) << 16);

        result.push(ENCODE_LUT[word as u8]);
        result.push(ENCODE_LUT[(word >> 6) as u8]);
        result.push(ENCODE_LUT[(word >> 12) as u8]);
        result.push(ENCODE_LUT[(word >> 18) as u8]);
    }

    // This is faster than matching on chunks.remainder().len():
    let mut word: u32 = 0;
    let mut word_bitlen: u32 = 0;
    for &byte in chunks.remainder() {
        word += u32::from(byte) << word_bitlen;
        word_bitlen += 8;
    }
    while word_bitlen > 0 {
        result.push(ENCODE_LUT[word as u8]);
        word >>= 6;
        word_bitlen = word_bitlen.saturating_sub(6);
    }

    // SAFETY: ENCODE_LUT contains only ASCII symbols
    unsafe { String::from_utf8_unchecked(result) }
}

#[cfg(feature = "unsafe")]
// About 103% faster
pub(crate) fn encode(data: &[u8]) -> String {
    let mut result: Vec<u8> = Vec::with_capacity((data.len() * 4 + 2) / 3);
    let mut chunks = data.chunks_exact(3);

    let mut len = 0;
    let mut ptr = result.as_mut_ptr();
    for chunk in chunks.by_ref() {
        len += 4;

        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        // SAFETY: the Vec should be allocated with sufficient capacity
        unsafe {
            ptr.write(ENCODE_LUT[b0]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[((b0 >> 6) | (b1 << 2))]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[((b1 >> 4) | (b2 << 4))]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[(b2 >> 2)]);
            ptr = ptr.add(1);
        }
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        2 => {
            len += 3;
            let b0 = remainder[0];
            let b1 = remainder[1];

            // SAFETY: the Vec should be allocated with sufficient capacity
            unsafe {
                ptr.write(ENCODE_LUT[b0]);
                ptr = ptr.add(1);
                ptr.write(ENCODE_LUT[((b0 >> 6) | (b1 << 2))]);
                ptr = ptr.add(1);
                ptr.write(ENCODE_LUT[(b1 >> 4)]);
            }
        }
        1 => {
            len += 2;
            let b0 = remainder[0];

            // SAFETY: the Vec should be allocated with sufficient capacity
            unsafe {
                ptr.write(ENCODE_LUT[b0]);
                ptr = ptr.add(1);
                ptr.write(ENCODE_LUT[(b0 >> 6)]);
            }
        }
        _ => (),
    }

    // SAFETY: ENCODE_LUT contains only ASCII symbols
    unsafe {
        result.set_len(len);
        String::from_utf8_unchecked(result)
    }
}
