use crate::register::Register;

/// Flag for register which can be used as memory address.
pub trait RegisterAddress: Register {}