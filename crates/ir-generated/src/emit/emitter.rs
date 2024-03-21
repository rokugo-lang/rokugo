use std::mem;

use rokugo_ir::{
    container::IrContainer,
    op_code::IrOpCode,
    register::{chill::RegisterChill, natural::RegisterNat32, Register, RegisterId},
};

use crate::errors::register::RegisterAllocationError;

use super::register_allocator::{RegisterAllocator, RegisterDropGuard};

pub struct IrEmitter {
    data: Vec<u8>,
    register_allocator: RegisterAllocator,
}

impl IrEmitter {
    pub fn new() -> Self {
        IrEmitter {
            data: Vec::new(),
            register_allocator: RegisterAllocator::new(),
        }
    }
}

/// # Local Memory
impl IrEmitter {
    /// Marks which virtual registers are most optimal to [chill][rokugo_ir::register::chill] in the current context.
    pub fn mark_register_chill(&mut self, registers_to_chill: RegisterChill) -> &mut Self {
        self.emit(IrOpCode::MarkRegisterChill);
        self.data
            .extend_from_slice(&registers_to_chill.to_le_bytes());
        self
    }

    /// Allocates a virtual register, or prepare a native register to store a new 32-bit natural value.
    pub fn alloc_register_nat32(
        &mut self,
    ) -> Result<RegisterDropGuard<RegisterNat32>, RegisterAllocationError> {
        let id = self.register_allocator.next_nat32()?;
        self.emit(IrOpCode::AllocRegisterNat32);
        self.emit_register_id(id);

        // SAFETY: This is safe, because this `id` is allocated with expected type by `RegisterAllocator`.
        Ok(RegisterDropGuard::new(unsafe {
            RegisterNat32::new_unchecked(id)
        }))
    }

    /// Drops a virtual register, what preverts it from being chilled.
    ///
    /// # Remarks
    ///
    /// This instruction is not dropping any memory like pointer etc. It is only a hint to the JIT compiler.
    pub fn drop_register(&mut self, register: RegisterDropGuard<impl Register>) -> &mut Self {
        self.register_allocator.drop(register.id());
        #[cfg(debug_assertions)]
        mem::forget(register);
        self
    }

    /// Loads 32-bit natural literal into register.
    pub fn load_nat32(&mut self, register: &RegisterNat32, value: u32) -> &mut Self {
        self.emit(IrOpCode::LoadNat32);
        self.emit_register_id(register.id());
        self.emit_nat32(value);
        self
    }
}

/// # Local
impl IrEmitter {
    fn emit(&mut self, op_code: IrOpCode) {
        self.data.extend_from_slice(&(op_code as u16).to_le_bytes());
    }

    fn emit_register_id(&mut self, register_id: RegisterId) {
        self.data.extend_from_slice(&register_id.to_le_bytes());
    }

    fn emit_nat32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }
}

impl Default for IrEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<IrEmitter> for IrContainer {
    fn from(ir_emitter: IrEmitter) -> Self {
        unsafe { IrContainer::from_vec(ir_emitter.data) }
    }
}
