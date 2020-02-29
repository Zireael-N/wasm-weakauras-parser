mod scalar;

pub(crate) fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    scalar::decode(s)
}
