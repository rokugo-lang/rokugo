use std::{mem, ops::Range};

use bytemuck::Pod;
use rokugo_common::color::ColoredDisplay;

use super::op_code::{MirInstruction, MirInstructionData, MirInstructionMeta, MirOpCode};

#[derive(Debug)]
pub struct MirContainer {
    pub(super) data: Vec<u8>,
}

impl MirContainer {
    pub fn iter(&self) -> MirContainerIterator {
        MirContainerIterator {
            content: self,
            index: 0,
        }
    }

    pub(super) fn emit_native_bytes(&mut self, value: impl Pod) {
        let bytes = bytemuck::bytes_of(&value);
        self.data.extend_from_slice(bytes);
    }
}

impl<'c> IntoIterator for &'c MirContainer {
    type Item = MirInstruction<'c>;
    type IntoIter = MirContainerIterator<'c>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct MirContainerIterator<'container> {
    content: &'container MirContainer,
    index: usize,
}

impl<'container> MirContainerIterator<'container> {
    unsafe fn read_native<T>(&mut self) -> T {
        let mut value = mem::MaybeUninit::<T>::uninit();
        let ptr = value.as_mut_ptr() as *mut u8;
        ptr.copy_from_nonoverlapping(
            self.content.data.as_ptr().byte_add(self.index),
            mem::size_of::<T>(),
        );
        self.index += mem::size_of::<T>();
        value.assume_init()
    }

    unsafe fn read_native_slice<T>(&mut self, count: usize) -> &'container [T] {
        let ptr = self.content.data.as_ptr().byte_add(self.index) as *const T;
        let slice = std::slice::from_raw_parts(ptr, count);
        self.index += mem::size_of::<T>() * count;
        slice
    }

    unsafe fn read_instruction(
        &mut self,
        meta: &mut MirInstructionMeta,
    ) -> Option<MirInstructionData<'container>> {
        let op_code: MirOpCode = self.read_native();
        match op_code {
            // ! Memory
            MirOpCode::DefineNat32 => Some(MirInstructionData::DefineNat32(
                self.read_native(),
                self.read_native(),
            )),
            MirOpCode::DefineInt32 => Some(MirInstructionData::DefineInt32(
                self.read_native(),
                self.read_native(),
            )),
            // ! Control flow
            MirOpCode::ReturnValue => Some(MirInstructionData::ReturnValue(self.read_native())),
            MirOpCode::Call => {
                let result = self.read_native();
                let function_id = self.read_native();
                let arguments_count: u8 = self.read_native();
                let arguments = self.read_native_slice(arguments_count as usize);

                Some(MirInstructionData::Call(result, function_id, arguments))
            }
            // ! Meta
            MirOpCode::MetaSpan => {
                meta.span = Some(Range {
                    start: self.read_native(),
                    end: self.read_native(),
                });
                None
            }
        }
    }
}

impl<'container> Iterator for MirContainerIterator<'container> {
    type Item = MirInstruction<'container>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.content.data.len() {
            return None;
        }

        let mut meta = MirInstructionMeta::default();
        loop {
            if let Some(data) = unsafe { self.read_instruction(&mut meta) } {
                return Some(MirInstruction { data, meta });
            }
        }
    }
}

impl ColoredDisplay for MirContainer {
    fn fmt_with_color(&self, f: &mut dyn termcolor::WriteColor) -> std::io::Result<()> {
        for instruction in self.iter() {
            instruction.fmt_with_color(f)?;
        }
        f.reset()?;
        Ok(())
    }
}
