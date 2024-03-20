use std::{error::Error, fmt};

#[derive(Debug)]
pub enum IrInstructionReadError {
    InvalidOpCode(usize, u16),
}

impl Error for IrInstructionReadError {}

impl fmt::Display for IrInstructionReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrInstructionReadError::InvalidOpCode(index, op_code) => {
                write!(f, "Invalid op code `{}` at index: {}", op_code, index)
            }
        }
    }
}
