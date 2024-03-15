use std::mem;

use super::{
    traits::naturals::{RegisterN, RegisterN16, RegisterN32, RegisterN64, RegisterN8},
    Register, RegisterId, X_START_INDEX,
};

/// # General Purpose Registers
#[repr(u8)]
pub enum RegisterX {
    R0 = X_START_INDEX,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
}

impl Register for RegisterX {
    fn id(&self) -> RegisterId {
        unsafe { mem::transmute_copy(self) }
    }
}

impl RegisterN for RegisterX {}
impl RegisterN64 for RegisterX {}
impl RegisterN32 for RegisterX {}
impl RegisterN16 for RegisterX {}
impl RegisterN8 for RegisterX {}
