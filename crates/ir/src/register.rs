use self::{general_purpose::RegisterX, special::RegisterSP};

pub mod general_purpose;
pub mod special;
pub mod traits;

pub(super) const X_START_INDEX: u8 = 0;
pub(super) const S_START_INDEX: u8 = 128;

pub struct RegisterId(u8);

pub const X0: RegisterX = RegisterX::R0;
pub const X1: RegisterX = RegisterX::R1;
pub const X2: RegisterX = RegisterX::R2;
pub const X3: RegisterX = RegisterX::R3;
pub const X4: RegisterX = RegisterX::R4;
pub const X5: RegisterX = RegisterX::R5;
pub const X6: RegisterX = RegisterX::R6;
pub const X7: RegisterX = RegisterX::R7;
pub const X8: RegisterX = RegisterX::R8;
pub const X9: RegisterX = RegisterX::R9;
pub const X10: RegisterX = RegisterX::R10;
pub const X11: RegisterX = RegisterX::R11;
pub const X12: RegisterX = RegisterX::R12;
pub const X13: RegisterX = RegisterX::R13;
pub const X14: RegisterX = RegisterX::R14;

/// Stack pointer
pub const SP: RegisterSP = RegisterSP {};
