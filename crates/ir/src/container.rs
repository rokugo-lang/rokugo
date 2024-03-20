use std::mem;

use crate::{
    instruction_read_error::IrInstructionReadError,
    op_code::{IrInstruction, IrOpCode},
    register::{chill::RegisterChill, RegisterId},
};

pub struct IrContainer {
    data: Vec<u8>,
}

impl IrContainer {
    /// # Safety
    /// This function can receive any data and it is up to the caller to ensure that the data is valid IR in valid
    /// version.
    pub unsafe fn from_vec(data: Vec<u8>) -> Self {
        IrContainer { data }
    }

    pub fn iter(&self) -> IrContainerIterator {
        IrContainerIterator {
            container: self,
            index: 0,
        }
    }
}

impl<'c> IntoIterator for &'c IrContainer {
    type Item = Result<IrInstruction<'c>, IrInstructionReadError>;
    type IntoIter = IrContainerIterator<'c>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IrContainerIterator<'c> {
    container: &'c IrContainer,
    index: usize,
}

impl<'c> IrContainerIterator<'c> {
    /// # Safety
    /// Caller must ensure that the data is valid IR in valid version.
    unsafe fn read_instruction(&mut self) -> Result<IrInstruction<'c>, IrInstructionReadError> {
        let op_code = match IrOpCode::try_from(self.read_nat16()) {
            Ok(op_code) => op_code,
            Err(err) => {
                return Err(IrInstructionReadError::InvalidOpCode(
                    self.index - mem::size_of::<IrOpCode>(),
                    err.number,
                ))
            }
        };
        match op_code {
            // ! Local Memory
            IrOpCode::AllocRegisterNat32 => {
                const REGISTER_CHILL_SIZE: usize = mem::size_of::<RegisterChill>();
                Ok(IrInstruction::AllocRegisterNat32(
                    self.read_register_id(),
                    RegisterChill::from_le_bytes(&self.read_byte_array::<REGISTER_CHILL_SIZE>()),
                ))
            }
            IrOpCode::DropRegister => Ok(IrInstruction::DropRegister(self.read_register_id())),
            IrOpCode::LoadNat32 => Ok(IrInstruction::LoadNat32(
                self.read_register_id(),
                self.read_nat32(),
            )),
        }
    }

    fn read_byte_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        let array = self.container.data[self.index..self.index + LENGTH]
            .try_into()
            .unwrap();
        self.index += LENGTH;
        array
    }

    /// # Safety
    /// Caller must ensure that the next bytes in data is a valid [`RegisterId`].
    unsafe fn read_register_id(&mut self) -> RegisterId {
        mem::transmute(self.read_nat16())
    }

    /// # Safety
    /// Caller must ensure that the next bytes in data is a valid [`u32`].
    unsafe fn read_nat32(&mut self) -> u32 {
        u32::from_le_bytes(self.read_byte_array())
    }

    /// # Safety
    /// Caller must ensure that the next bytes in data is a valid [`u16`].
    unsafe fn read_nat16(&mut self) -> u16 {
        u16::from_le_bytes(self.read_byte_array())
    }
}

impl<'c> Iterator for IrContainerIterator<'c> {
    type Item = Result<IrInstruction<'c>, IrInstructionReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.container.data.len() {
            // SAFETY: `IrContainerIterator` can be created only from `IrContainer` which forces ensuring data to be
            // valid IR in current version.
            Some(unsafe { self.read_instruction() })
        } else {
            None
        }
    }
}
