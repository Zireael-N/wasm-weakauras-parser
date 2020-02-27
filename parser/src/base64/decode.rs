// Slightly modified version of:
// MODP_B64 - High performance base64 encoder/decoder
// Copyright (C) 2005-2016 Nick Galbreath
// https://github.com/client9/stringencoders

use super::byte_map::{BAD_SYMBOL, DECODE_LUT0, DECODE_LUT1, DECODE_LUT2, DECODE_LUT3};

const INVALID_B64: &str = "failed to decode base64";

pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let len = s.len();
    let mut result = Vec::with_capacity(len * 3 / 4);

    let mut chunks = s.as_bytes().chunks_exact(4);

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
            result.push(word[0]);
            result.push(word[1]);
            result.push(word[2]);
        } else {
            result.push(word[3]);
            result.push(word[2]);
            result.push(word[1]);
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
                result.push(word[0]);
                result.push(word[1]);
            } else {
                result.push(word[3]);
                result.push(word[2]);
            }
        }
        2 => {
            let word = DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]];

            if word == BAD_SYMBOL {
                return Err(INVALID_B64);
            }

            if cfg!(target_endian = "little") {
                result.push(word as u8);
            } else {
                result.push(word.to_ne_bytes()[3]);
            }
        }
        1 => {
            if DECODE_LUT0[remainder[0]] == BAD_SYMBOL {
                return Err(INVALID_B64);
            }
        }
        _ => (),
    }

    Ok(result)
}
