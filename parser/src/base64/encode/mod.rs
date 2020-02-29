mod scalar;

pub(crate) fn encode(data: &[u8]) -> String {
    scalar::encode(data)
}
