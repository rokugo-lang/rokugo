use crate::register::RegisterId;

#[derive(Debug)]
pub enum IrOpCode {
    // ! Local Memory
    /// Loads 32-bit natural literal into register.
    /// # Layout
    /// - [`RegisterId`] - destination register
    /// - [`u32`] - literal value
    LoadNat32,
}

pub enum IrInstruction<'container> {
    // ! Local Memory
    LoadNat32(RegisterId, u32),
    Call(&'container [u8]),
}
