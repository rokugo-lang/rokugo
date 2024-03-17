use super::{Register, RegisterId};

/// Flag for any size natural register.
pub trait RegisterNat: Register {}

/// 64-bit natural register.
pub struct RegisterNat64(RegisterId);

impl RegisterNat for RegisterNat64 {}
impl Register for RegisterNat64 {
    fn id(&self) -> RegisterId {
        self.0
    }
}

/// 32-bit natural register.
pub struct RegisterNat32(RegisterId);

impl RegisterNat for RegisterNat32 {}
impl Register for RegisterNat32 {
    fn id(&self) -> RegisterId {
        self.0
    }
}

/// 16-bit natural register.
pub struct RegisterNat16(RegisterId);

impl RegisterNat for RegisterNat16 {}
impl Register for RegisterNat16 {
    fn id(&self) -> RegisterId {
        self.0
    }
}

/// 8-bit natural register.
pub struct RegisterNat8(RegisterId);

impl RegisterNat for RegisterNat8 {}
impl Register for RegisterNat8 {
    fn id(&self) -> RegisterId {
        self.0
    }
}
