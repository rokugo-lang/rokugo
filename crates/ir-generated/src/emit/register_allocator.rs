use std::{
    mem,
    ops::{Deref, Range},
};

use rokugo_ir::register::{self, Register, RegisterId};

use crate::errors::register::RegisterAllocationError;

pub struct RegisterDropGuard<T>
where
    T: Register,
{
    register: T,
}

impl<T> RegisterDropGuard<T>
where
    T: Register,
{
    pub(crate) fn new(register: T) -> Self {
        Self { register }
    }
}

impl<T> Deref for RegisterDropGuard<T>
where
    T: Register,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.register
    }
}

#[cfg(debug_assertions)]
impl<T> Drop for RegisterDropGuard<T>
where
    T: Register,
{
    fn drop(&mut self) {
        panic!("Register was dropped without DropRegister instruction")
    }
}

pub(crate) struct RegisterAllocator {
    register_id_nat32: RegisterIdAllocator,
    dropped_registers: Vec<RegisterId>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            register_id_nat32: RegisterIdAllocator::new(register::NAT32_ID_RANGE.start),
            dropped_registers: Vec::new(),
        }
    }

    pub fn drop(&mut self, register_id: RegisterId) {
        self.dropped_registers.push(register_id)
    }

    pub fn next_nat32(&mut self) -> Result<RegisterId, RegisterAllocationError> {
        match self.get_dropped(register::NAT32_ID_RANGE) {
            Some(r) => Ok(r),
            None => self.register_id_nat32.next(register::NAT32_ID_RANGE.end),
        }
    }

    fn get_dropped(&mut self, range: Range<u16>) -> Option<RegisterId> {
        if let Some(index) = self.dropped_registers.iter().position(|x| {
            let unwrapped = x.into_inner();
            unwrapped >= range.start && unwrapped < range.end
        }) {
            Some(self.dropped_registers.remove(index))
        } else {
            None
        }
    }
}

pub(crate) struct RegisterIdAllocator {
    next_register_id: u16,
}

impl RegisterIdAllocator {
    pub(crate) fn new(range_start: u16) -> Self {
        Self {
            next_register_id: range_start,
        }
    }

    pub(crate) fn next(&mut self, range_end: u16) -> Result<RegisterId, RegisterAllocationError> {
        let id = self.next_register_id;
        if id >= range_end {
            return Err(RegisterAllocationError {
                last_valid_register_id: id,
            });
        }
        self.next_register_id += 1;

        // SAFETY: This is safe, because `RegisterId` is a wrapper around `u16`.
        Ok(unsafe { mem::transmute(id) })
    }
}
