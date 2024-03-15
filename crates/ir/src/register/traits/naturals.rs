use super::Register;

/// Flag for register which can be used as unspecified bit-width natural.
pub trait RegisterN: Register {}

/// Flag for register which can be used as platform bit-width natural.
pub trait RegisterNSize: RegisterN {}

/// Flag for register which can be used as 64-bit natural.
pub trait RegisterN64: RegisterN {}

/// Flag for register which can be used as 32-bit natural.
pub trait RegisterN32: RegisterN {}

/// Flag for register which can be used as 16-bit natural.
pub trait RegisterN16: RegisterN {}

/// Flag for register which can be used as 8-bit natural.
pub trait RegisterN8: RegisterN {}
