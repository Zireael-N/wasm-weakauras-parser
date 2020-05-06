#![no_main]
use libfuzzer_sys::fuzz_target;
use weakauras_parser::base64;

fuzz_target!(|data: &[u8]| {
    if let Ok(encoded) = base64::encode_raw(data) {
        let decoded = base64::decode(&encoded).expect("Failed to decode what we've encoded");
        assert!(decoded == data, "Decoded data differs from the input");
    }
});
