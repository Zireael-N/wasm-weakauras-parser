#[cfg(all(test, target_feature = "avx2"))]
mod avx2;
mod scalar;
#[cfg(all(any(feature = "unsafe", test), target_feature = "ssse3"))]
mod sse;

#[cfg(all(feature = "unsafe", target_feature = "ssse3"))]
pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let len = s.len();
    if len % 4 == 1 {
        return Err("invalid base64 length");
    }

    let mut buffer = Vec::with_capacity(len * 3 / 4);
    unsafe { sse::decode(s.as_bytes(), &mut buffer)? }
    Ok(buffer)
}

#[cfg(all(feature = "unsafe", not(target_feature = "ssse3")))]
pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let len = s.len();
    if len % 4 == 1 {
        return Err("invalid base64 length");
    }

    let mut buffer = Vec::with_capacity(len * 3 / 4);
    unsafe {
        scalar::decode(s.as_bytes(), &mut buffer)?;
    }
    Ok(buffer)
}

#[cfg(not(feature = "unsafe"))]
pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    let len = s.len();
    if len % 4 == 1 {
        return Err("invalid base64 length");
    }

    let mut buffer = Vec::with_capacity(len * 3 / 4);
    scalar::decode(s.as_bytes(), &mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_and_sse_return_same_values() {
        let data: Vec<u8> = (b'0'..=b'9')
            .chain(b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .chain(b'('..=b')')
            .cycle()
            .take(1024 * 1024 + 3)
            .collect();

        let cap = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(cap);
        let mut buf2 = Vec::with_capacity(cap);

        unsafe {
            scalar::decode(&data, &mut buf1).unwrap();
            sse::decode(&data, &mut buf2).unwrap();
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn scalar_and_avx2_return_same_values() {
        if !is_x86_feature_detected!("avx2") {
            panic!("AVX2 support is not detected");
        }

        let data: Vec<u8> = (b'0'..=b'9')
            .chain(b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .chain(b'('..=b')')
            .cycle()
            .take(1024 * 1024 + 3)
            .collect();

        let cap = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(cap);
        let mut buf2 = Vec::with_capacity(cap);

        unsafe {
            scalar::decode(&data, &mut buf1).unwrap();
            avx2::decode(&data, &mut buf2).unwrap();
        }

        assert_eq!(buf1, buf2);
    }
}
