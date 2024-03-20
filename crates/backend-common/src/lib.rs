// move to rokugo-ir crate?

use std::fmt::Display;

use bytemuck::{Pod, Zeroable};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Zeroable, Pod)]
pub struct ValueId(u32);

impl Display for ValueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Zeroable, Pod)]
pub struct FunctionId(u64);

impl Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Zeroable, Pod)]
pub struct UnstableTypeId(u64);

impl UnstableTypeId {
    /// The type of a value that is nothing.
    pub const VOID: Self = Self(0);
    /// The 32-bit natural type.
    pub const NAT32: Self = Self(1);
}
