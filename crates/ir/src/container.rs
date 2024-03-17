use std::mem;

use crate::{
    op_code::{IrInstruction, IrOpCode},
    register::{chill::RegisterChill, RegisterId},
};

pub struct IrContainer {
    _data: Vec<u8>,
}

impl IrContainer {
    /// # Safety
    /// This function can receive any data and it is up to the caller to ensure that the data is valid IR in valid
    /// version.
    pub unsafe fn from_vec(data: Vec<u8>) -> Self {
        IrContainer { _data: data }
    }

    pub fn iter(&self) -> IrContainerIterator {
        IrContainerIterator {
            container: self,
            index: 0,
        }
    }
}

impl<'container> IntoIterator for &'container IrContainer {
    type Item = IrInstruction<'container>;
    type IntoIter = IrContainerIterator<'container>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IrContainerIterator<'container> {
    container: &'container IrContainer,
    index: usize,
}

impl<'container> IrContainerIterator<'container> {
    /// # Safety
    /// Caller must ensure that the data is valid IR in valid version.
    unsafe fn read_instruction(&mut self) -> IrInstruction<'container> {
        match self.read_op_code() {
            // ! Local Memory
            IrOpCode::AllocRegisterNat32 => {
                const REGISTER_CHILL_SIZE: usize = mem::size_of::<RegisterChill>();
                IrInstruction::AllocRegisterNat32(
                    self.read_register_id(),
                    RegisterChill::from_le_bytes(&self.read_byte_array::<REGISTER_CHILL_SIZE>()),
                )
            }
            IrOpCode::DropRegister => IrInstruction::DropRegister(self.read_register_id()),
            IrOpCode::LoadNat32 => {
                IrInstruction::LoadNat32(self.read_register_id(), self.read_nat32())
            }
        }
    }

    fn read_byte_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        let array = self.container._data[self.index..self.index + LENGTH]
            .try_into()
            .unwrap();
        self.index += LENGTH;
        array
    }

    /// # Safety
    /// Caller must ensure that the next bytes in data is a valid [`IrOpCode`].
    unsafe fn read_op_code(&mut self) -> IrOpCode {
        mem::transmute(u16::from_le_bytes(self.read_byte_array()))
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

impl<'container> Iterator for IrContainerIterator<'container> {
    type Item = IrInstruction<'container>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.container._data.len() {
            // SAFETY: `IrContainerIterator` can be created only from `IrContainer` which forces ensuring data to be
            // valid IR in current version.
            Some(unsafe { self.read_instruction() })
        } else {
            None
        }
    }
}
