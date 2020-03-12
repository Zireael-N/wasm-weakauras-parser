// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright (c) 2015-2016, Wojciech Muła, Alfred Klomp, Daniel Lemire
// All rights reserved.
// Licensed under BSD 2-Clause (see LICENSES/fastbase64)

use super::scalar;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(all(test, target_feature = "avx2"))]
#[cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::cast_ptr_alignment, clippy::unreadable_literal)
)]
#[inline(always)]
/// SAFETY: the caller must ensure that buf can hold AT LEAST ((s.len() * 4 + 2) / 3) more elements
pub(crate) unsafe fn encode(s: &[u8], buf: &mut String) {
    let mut len = s.len();
    let mut out_len = buf.len();

    let mut ptr = s.as_ptr();
    let mut out_ptr = buf[out_len..].as_mut_ptr();

    let shuf = _mm256_set_epi8(
        10, 9, 11, 10, 7, 6, 8, 7, 4, 3, 5, 4, 1, 0, 2, 1, 10, 9, 11, 10, 7, 6, 8, 7, 4, 3, 5, 4,
        1, 0, 2, 1,
    );

    while len >= 32 {
        let lo = _mm_loadu_si128(ptr as *const _);
        let hi = _mm_loadu_si128(ptr.add(12) as *const _);

        let src = _mm256_shuffle_epi8(_mm256_set_m128i(hi, lo), shuf);

        let t1 = _mm256_mullo_epi16(
            _mm256_and_si256(src, _mm256_set1_epi32(0x003f03f0)),
            _mm256_set1_epi32(0x01000010),
        );
        let t2 = _mm256_mulhi_epu16(
            _mm256_and_si256(src, _mm256_set1_epi32(0x0fc0fc00)),
            _mm256_set1_epi32(0x04000040),
        );

        let indices = _mm256_shuffle_epi8(
            _mm256_or_si256(t1, t2),
            _mm256_set_epi8(
                12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11,
                4, 5, 6, 7, 0, 1, 2, 3,
            ),
        );

        let mut result = _mm256_or_si256(
            _mm256_subs_epu8(indices, _mm256_set1_epi8(51)),
            _mm256_and_si256(
                _mm256_cmpgt_epi8(_mm256_set1_epi8(26), indices),
                _mm256_set1_epi8(13),
            ),
        );

        let offsets = _mm256_setr_epi8(
            39, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -22, -22, 97, 0, 0, 39, -4, -4, -4, -4, -4,
            -4, -4, -4, -4, -4, -22, -22, 97, 0, 0,
        );

        result = _mm256_add_epi8(_mm256_shuffle_epi8(offsets, result), indices);

        _mm256_storeu_si256(out_ptr as *mut _, result);
        out_ptr = out_ptr.add(32);
        out_len += 32;

        len -= 24;
        ptr = ptr.add(24);
    }
    buf.as_mut_vec().set_len(out_len);

    scalar::encode(core::slice::from_raw_parts(ptr, len), buf)
}
