use core::ops::{Index, IndexMut};

pub struct ByteMap([u8; 256]);

impl ByteMap {
    #[inline(always)]
    pub fn get(&self, byte: u8) -> &u8 {
        // safety: this is safe because the underlying array has 256 elements in it
        unsafe { self.0.get_unchecked(byte as usize) }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, byte: u8) -> &mut u8 {
        // safety: this is safe because the underlying array has 256 elements in it
        unsafe { self.0.get_unchecked_mut(byte as usize) }
    }
}

impl Index<u8> for ByteMap {
    type Output = u8;

    #[inline(always)]
    fn index(&self, byte: u8) -> &Self::Output {
        self.get(byte)
    }
}

impl IndexMut<u8> for ByteMap {
    #[inline(always)]
    fn index_mut(&mut self, byte: u8) -> &mut Self::Output {
        self.get_mut(byte)
    }
}

#[rustfmt::skip]
pub static LOOKUP_TABLE: ByteMap = ByteMap([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    62, 63, // '(', ')'
    0, 0, 0, 0, 0, 0,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, // '0' - '9'
    0, 0, 0, 0, 0, 0, 0,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, // 'A' - 'Z'
    0, 0, 0, 0, 0, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, // 'a' - 'z'
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
