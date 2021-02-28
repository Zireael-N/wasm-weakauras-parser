#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz this with a timeout, there's an infinite loop.
fuzz_target!(|data: &[u8]| {
    let _ = weakauras_parser_huffman::decompress(data);
});
