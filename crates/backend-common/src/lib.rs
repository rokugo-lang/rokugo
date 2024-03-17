// move to rokugo-ir crate?

use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ValueId(u32);

impl Display for ValueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FunctionId(u64);

impl Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnstableTypeId(u64);

impl UnstableTypeId {
    /// The type of a value that is nothing.
    pub const VOID: Self = Self(0);
    /// The 32-bit natural type.
    pub const NAT32: Self = Self(1);
}
