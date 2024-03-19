use std::ops::Range;

pub mod chill;
pub mod natural;
pub mod special;

const RANGE: u16 = 1024;
const fn id_range(index: u16) -> Range<u16> {
    (RANGE * index)..(RANGE * (index + 1))
}

// Natural
pub const NAT64_ID_RANGE: Range<u16> = id_range(0);
pub const NAT32_ID_RANGE: Range<u16> = id_range(1);
pub const NAT16_ID_RANGE: Range<u16> = id_range(2);
pub const NAT8_ID_RANGE: Range<u16> = id_range(3);

// Integer
pub const INT64_ID_RANGE: Range<u16> = id_range(4);
pub const INT32_ID_RANGE: Range<u16> = id_range(5);
pub const INT16_ID_RANGE: Range<u16> = id_range(6);
pub const INT8_ID_RANGE: Range<u16> = id_range(7);

// Float
pub const FLOAT64_ID_RANGE: Range<u16> = id_range(8);
pub const FLOAT32_ID_RANGE: Range<u16> = id_range(9);
pub const FLOAT16_ID_RANGE: Range<u16> = id_range(10);

// Misc
pub const SPECIAL_ID_RANGE: Range<u16> = id_range(11);
pub const PTR_ID_RANGE: Range<u16> = id_range(12);
pub const BIT_ID_RANGE: Range<u16> = id_range(13);

pub trait Register {
    fn id(&self) -> RegisterId;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegisterId(u16);

impl RegisterId {
    /// Converts the register id to little-endian bytes.
    pub fn to_le_bytes(self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    /// Unwraps the register id to internal value.
    pub fn into_inner(self) -> u16 {
        self.0
    }
}
