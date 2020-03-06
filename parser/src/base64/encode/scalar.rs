use crate::base64::byte_map::ENCODE_LUT;

#[cfg(not(feature = "unsafe"))]
#[inline(always)]
pub(crate) fn encode(data: &[u8], buf: &mut String) {
    // SAFETY: we'll be pushing ASCII symbols
    let buf = unsafe { buf.as_mut_vec() };
    let mut chunks = data.chunks_exact(3);

    for chunk in chunks.by_ref() {
        let word = u32::from(chunk[0]) + (u32::from(chunk[1]) << 8) + (u32::from(chunk[2]) << 16);

        buf.push(ENCODE_LUT[word as u8]);
        buf.push(ENCODE_LUT[(word >> 6) as u8]);
        buf.push(ENCODE_LUT[(word >> 12) as u8]);
        buf.push(ENCODE_LUT[(word >> 18) as u8]);
    }

    // This is faster than matching on chunks.remainder().len():
    let mut word: u32 = 0;
    let mut word_bitlen: u32 = 0;
    for &byte in chunks.remainder() {
        word += u32::from(byte) << word_bitlen;
        word_bitlen += 8;
    }
    while word_bitlen > 0 {
        buf.push(ENCODE_LUT[word as u8]);
        word >>= 6;
        word_bitlen = word_bitlen.saturating_sub(6);
    }
}

// About 103% faster
#[cfg(feature = "unsafe")]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
pub(crate) unsafe fn encode(data: &[u8], buf: &mut String) {
    let mut chunks = data.chunks_exact(3);

    let mut len = buf.len();
    let mut ptr = buf[len..].as_mut_ptr();
    for chunk in chunks.by_ref() {
        len += 4;

        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        ptr.write(ENCODE_LUT[b0]);
        ptr = ptr.add(1);
        ptr.write(ENCODE_LUT[((b0 >> 6) | (b1 << 2))]);
        ptr = ptr.add(1);
        ptr.write(ENCODE_LUT[((b1 >> 4) | (b2 << 4))]);
        ptr = ptr.add(1);
        ptr.write(ENCODE_LUT[(b2 >> 2)]);
        ptr = ptr.add(1);
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        2 => {
            len += 3;
            let b0 = remainder[0];
            let b1 = remainder[1];

            ptr.write(ENCODE_LUT[b0]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[((b0 >> 6) | (b1 << 2))]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[(b1 >> 4)]);
        }
        1 => {
            len += 2;
            let b0 = remainder[0];

            ptr.write(ENCODE_LUT[b0]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[(b0 >> 6)]);
        }
        _ => (),
    }

    buf.as_mut_vec().set_len(len);
}
