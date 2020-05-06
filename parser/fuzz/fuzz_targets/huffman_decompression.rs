#![no_main]
use libfuzzer_sys::fuzz_target;
use weakauras_parser::huffman;

// Fuzz this with a timeout, there's an infinite loop.
fuzz_target!(|data: &[u8]| {
    let _ = huffman::decompress(data);
});
