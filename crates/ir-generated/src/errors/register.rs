use std::{error::Error, fmt, fmt::Display};

use rokugo_diagnostic::{note, Diagnostic, NoteKind, Severity};
use rokugo_ir::register;

#[derive(Debug)]
pub struct RegisterAllocationError {
    pub(crate) last_valid_register_id: u16,
}

impl From<&RegisterAllocationError> for Diagnostic {
    fn from(value: &RegisterAllocationError) -> Self {
        const NAT64: u16 = register::NAT64_ID_RANGE.end;
        const NAT32: u16 = register::NAT32_ID_RANGE.end;
        const NAT16: u16 = register::NAT16_ID_RANGE.end;
        const NAT8: u16 = register::NAT8_ID_RANGE.end;

        const INT64: u16 = register::INT64_ID_RANGE.end;
        const INT32: u16 = register::INT32_ID_RANGE.end;
        const INT16: u16 = register::INT16_ID_RANGE.end;
        const INT8: u16 = register::INT8_ID_RANGE.end;

        const FLOAT64: u16 = register::FLOAT64_ID_RANGE.end;
        const FLOAT32: u16 = register::FLOAT32_ID_RANGE.end;
        const FLOAT16: u16 = register::FLOAT16_ID_RANGE.end;

        const SPECIAL: u16 = register::SPECIAL_ID_RANGE.end;
        const PTR: u16 = register::PTR_ID_RANGE.end;
        const BIT: u16 = register::BIT_ID_RANGE.end;

        let register_type = match value.last_valid_register_id {
            NAT64 => "Nat64",
            NAT32 => "Nat32",
            NAT16 => "Nat16",
            NAT8 => "Nat8",
            INT64 => "Int64",
            INT32 => "Int32",
            INT16 => "Int16",
            INT8 => "Int8",
            FLOAT64 => "Float64",
            FLOAT32 => "Float32",
            FLOAT16 => "Float16",
            SPECIAL => "Special",
            PTR => "Pointer",
            BIT => "Bit",
            _ => "Unknown",
        };

        Severity::Bug
            .diagnostic(format!("register allocation failed. {register_type} ID range overflow"))
            .with_note(note(
                NoteKind::Note,
                "this can be caused if your function has too many variables; try factoring out your function to smaller ones"
            ))
    }
}

impl Error for RegisterAllocationError {}

impl Display for RegisterAllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Diagnostic::from(self))
    }
}
