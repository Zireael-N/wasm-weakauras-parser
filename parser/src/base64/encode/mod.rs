#[cfg(all(
    test,
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod avx2;
mod scalar;
#[cfg(all(
    any(feature = "unsafe", test),
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
mod sse;

#[inline(always)]
fn calculate_capacity(data: &[u8]) -> Result<usize, &'static str> {
    data.len()
        .checked_mul(4)
        .and_then(|len| len.checked_add(2))
        .map(|len| len / 3)
        .ok_or("cannot calculate capacity without overflowing")
}

#[cfg(all(
    feature = "unsafe",
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
pub(crate) fn encode(data: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_capacity(data)?);
    unsafe {
        sse::encode(data, &mut result);
    }
    Ok(result)
}

#[cfg(all(
    feature = "unsafe",
    any(
        not(any(target_arch = "x86", target_arch = "x86_64")),
        not(target_feature = "ssse3")
    )
))]
pub(crate) fn encode(data: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_capacity(data)?);
    unsafe {
        scalar::encode(data, &mut result);
    }
    Ok(result)
}

#[cfg(not(feature = "unsafe"))]
pub(crate) fn encode(data: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_capacity(data)?);
    scalar::encode(data, &mut result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_and_sse_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let cap = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(cap);
        let mut buf2 = String::with_capacity(cap);

        unsafe {
            scalar::encode(&data, &mut buf1);
            sse::encode(&data, &mut buf2);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    fn scalar_and_avx2_return_same_values() {
        if !is_x86_feature_detected!("avx2") {
            panic!("AVX2 support is not detected");
        }

        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let cap = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(cap);
        let mut buf2 = String::with_capacity(cap);

        unsafe {
            scalar::encode(&data, &mut buf1);
            avx2::encode(&data, &mut buf2);
        }

        assert_eq!(buf1, buf2);
    }
}
