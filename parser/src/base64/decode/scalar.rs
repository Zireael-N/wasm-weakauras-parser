// Slightly modified version of:
// MODP_B64 - High performance base64 encoder/decoder
// Copyright (C) 2005-2016 Nick Galbreath
// https://github.com/client9/stringencoders

use crate::base64::byte_map::{BAD_SYMBOL, DECODE_LUT0, DECODE_LUT1, DECODE_LUT2, DECODE_LUT3};

const INVALID_B64: &str = "failed to decode base64";

#[cfg(not(feature = "unsafe"))]
#[inline(always)]
pub(crate) fn decode(s: &[u8], buf: &mut Vec<u8>) -> Result<(), &'static str> {
    let mut chunks = s.chunks_exact(4);

    for chunk in chunks.by_ref() {
        let word = DECODE_LUT0[chunk[0]]
            | DECODE_LUT1[chunk[1]]
            | DECODE_LUT2[chunk[2]]
            | DECODE_LUT3[chunk[3]];

        if word == BAD_SYMBOL {
            return Err(INVALID_B64);
        }

        let word = word.to_ne_bytes();
        if cfg!(target_endian = "little") {
            buf.push(word[0]);
            buf.push(word[1]);
            buf.push(word[2]);
        } else {
            buf.push(word[3]);
            buf.push(word[2]);
            buf.push(word[1]);
        }
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        3 => {
            let word =
                DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]] | DECODE_LUT2[remainder[2]];

            if word == BAD_SYMBOL {
                return Err(INVALID_B64);
            }

            let word = word.to_ne_bytes();
            if cfg!(target_endian = "little") {
                buf.push(word[0]);
                buf.push(word[1]);
            } else {
                buf.push(word[3]);
                buf.push(word[2]);
            }
        }
        2 => {
            let word = DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]];

            if word == BAD_SYMBOL {
                return Err(INVALID_B64);
            }

            if cfg!(target_endian = "little") {
                buf.push(word as u8);
            } else {
                buf.push(word.to_ne_bytes()[3]);
            }
        }
        _ => (),
    }

    Ok(())
}

// About 74% faster
#[cfg(feature = "unsafe")]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST (s.len() * 3 / 4) more elements
pub(crate) unsafe fn decode(s: &[u8], buf: &mut Vec<u8>) -> Result<(), &'static str> {
    let mut chunks = s.chunks_exact(4);

    let mut len = buf.len();
    let mut ptr = buf[len..].as_mut_ptr();

    for chunk in chunks.by_ref() {
        len += 3;

        let word = DECODE_LUT0[chunk[0]]
            | DECODE_LUT1[chunk[1]]
            | DECODE_LUT2[chunk[2]]
            | DECODE_LUT3[chunk[3]];

        if word == BAD_SYMBOL {
            return Err(INVALID_B64);
        }

        let word = word.to_ne_bytes();
        if cfg!(target_endian = "little") {
            ptr.write(word[0]);
            ptr = ptr.add(1);
            ptr.write(word[1]);
            ptr = ptr.add(1);
            ptr.write(word[2]);
            ptr = ptr.add(1);
        } else {
            ptr.write(word[3]);
            ptr = ptr.add(1);
            ptr.write(word[2]);
            ptr = ptr.add(1);
            ptr.write(word[1]);
            ptr = ptr.add(1);
        }
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        3 => {
            len += 2;

            let word =
                DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]] | DECODE_LUT2[remainder[2]];

            if word == BAD_SYMBOL {
                return Err(INVALID_B64);
            }

            let word = word.to_ne_bytes();
            if cfg!(target_endian = "little") {
                ptr.write(word[0]);
                ptr = ptr.add(1);
                ptr.write(word[1]);
            } else {
                ptr.write(word[3]);
                ptr = ptr.add(1);
                ptr.write(word[2]);
            }
        }
        2 => {
            len += 1;

            let word = DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]];

            if word == BAD_SYMBOL {
                return Err(INVALID_B64);
            }

            ptr.write(if cfg!(target_endian = "little") {
                word as u8
            } else {
                word.to_ne_bytes()[3]
            });
        }
        _ => (),
    }

    buf.set_len(len);

    Ok(())
}
