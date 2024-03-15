use std::mem;

use crate::{
    op_code::{IrInstruction, IrOpCode},
    register::RegisterId,
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
    unsafe fn read_instruction(&mut self) -> IrInstruction<'container> {
        match self.read_op_code() {
            IrOpCode::LoadNat32 => {
                IrInstruction::LoadNat32(self.read_register_id(), self.read_nat32())
            }
        }
    }

    unsafe fn read_byte_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        let array = self.container._data[self.index..self.index + LENGTH]
            .try_into()
            .unwrap();
        self.index += LENGTH;
        array
    }

    unsafe fn read_op_code(&mut self) -> IrOpCode {
        mem::transmute(u16::from_le_bytes(self.read_byte_array()))
    }

    unsafe fn read_register_id(&mut self) -> RegisterId {
        mem::transmute(self.read_nat8())
    }

    unsafe fn read_nat32(&mut self) -> u32 {
        u32::from_le_bytes(self.read_byte_array())
    }

    unsafe fn read_nat8(&mut self) -> u8 {
        let value = self.container._data[self.index];
        self.index += 1;
        value
    }
}

impl<'container> Iterator for IrContainerIterator<'container> {
    type Item = IrInstruction<'container>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.container._data.len() {
            Some(unsafe { self.read_instruction() })
        } else {
            None
        }
    }
}
