use num_enum::TryFromPrimitive;

use crate::register::{chill::RegisterChill, RegisterId};

#[derive(Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum IrOpCode {
    // ! Local Memory
    /// Allocates a virtual register, or prepare a native register to store a new 32-bit natural value.
    ///
    /// # Layout
    ///
    /// - [`RegisterId`] - defined register
    /// - [`RegisterChill`] - most optimal registers to [chill][crate::register::chill]
    AllocRegisterNat32,

    /// Drops a virtual register, what preverts it from being chilled.
    ///
    /// # Remarks
    ///
    /// This instruction is not dropping any memory like pointer etc. It is only a hint to the JIT compiler.
    ///
    /// # Layout
    ///
    /// - [`RegisterId`] - register to drop
    DropRegister,

    /// Loads 32-bit natural literal into register.
    ///
    /// # Layout
    ///
    /// - [`RegisterId`] - destination register
    /// - [`u32`] - literal value
    LoadNat32,
}

#[derive(Debug, PartialEq)]
pub enum IrInstruction<'container> {
    // ! Local Memory
    AllocRegisterNat32(RegisterId, RegisterChill),
    DropRegister(RegisterId),
    LoadNat32(RegisterId, u32),
    // ! Control Flow
    Call(&'container [u8]),
}
