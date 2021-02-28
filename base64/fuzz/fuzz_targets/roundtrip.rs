#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(encoded) = weakauras_parser_base64::encode_raw(data) {
        let decoded = weakauras_parser_base64::decode(&encoded).expect("Failed to decode what we've encoded");
        assert!(decoded == data, "Decoded data differs from the input");
    }
});
