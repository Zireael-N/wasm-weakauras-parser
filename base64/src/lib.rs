mod byte_map;
mod decode;
mod encode;
pub use decode::decode;
#[allow(unused_imports)]
pub use encode::{encode_raw, encode_with_prefix};

#[cfg(feature = "expose_internals")]
pub use decode::*;
#[cfg(feature = "expose_internals")]
pub use encode::*;
