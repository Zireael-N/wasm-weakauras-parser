#![no_main]
use libfuzzer_sys::fuzz_target;
use weakauras_parser::{deserialization::Deserializer, serialization::Serializer};

fuzz_target!(|data: &[u8]| {
    if let Ok(data) = std::str::from_utf8(data) {
        if let Ok(Some(value)) = Deserializer::from_str(data).deserialize_first() {
            // No reason to compare with the original data, because same numbers
            // can be encoded in different ways.
            assert!(Serializer::serialize(&value, None).is_ok(), "Couldn't serialize what we deserialized");
        }
    }
});
