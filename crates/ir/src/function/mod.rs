use std::fmt::Display;

use smallvec::SmallVec;

use crate::{container::IrContainer, r#type::UnstableTypeId, register::RegisterId};

pub mod collection;
pub mod load_error;

#[derive(Debug)]
#[non_exhaustive]
pub struct Function {
    pub signature: FunctionSignature,
    pub code: IrContainer,
}

impl Function {
    pub fn new(signature: FunctionSignature, code: IrContainer) -> Self {
        Self { signature, code }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct FunctionSignature {
    pub return_data: ReturnData,
}

impl FunctionSignature {
    pub fn new(return_data: ReturnData) -> Self {
        Self { return_data }
    }
}

#[derive(Debug)]
pub struct ReturnData {
    pub return_type: UnstableTypeId,
    pub container: ReturnDataContainer,
}

/// Store information how the return data is stored.
#[derive(Debug)]
pub enum ReturnDataContainer {
    /// Return unwrapped type in registers.
    Registers(SmallVec<[RegisterId; 4]>),
    /// Grows stack by size of return type, where store a return value.
    ///
    /// # Remarks
    ///
    /// Caller must remember about increased stack pointer, and return right data to their caller.
    Stack(),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnstableFunctionId(u64);

impl UnstableFunctionId {
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl Display for UnstableFunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)?;
        Ok(())
    }
}
