#![no_main]
use libfuzzer_sys::fuzz_target;
use weakauras_parser::lib_serialize::{Deserializer, Serializer};

fuzz_target!(|data: &[u8]| {
    if let Ok(Some(value)) = Deserializer::from_slice(data).deserialize_first() {
        assert!(Serializer::serialize(&value, None).is_ok(), "Couldn't serialize what we deserialized");
    }
});
