use std::mem;

use rokugo_ir::{
    container::IrContainer,
    op_code::IrOpCode,
    register::{traits::naturals::RegisterN32, RegisterId},
};

pub struct IrEmitter {
    data: Vec<u8>,
}

impl IrEmitter {
    pub fn new() -> Self {
        IrEmitter { data: Vec::new() }
    }
}

/// # Local Memory
impl IrEmitter {
    /// Loads 32-bit natural literal into register.
    pub fn load_nat32<R: RegisterN32>(&mut self, register: R, value: u32) -> &mut Self {
        unsafe {
            self.emit(IrOpCode::LoadNat32);
            self.emit_register_id(register.id());
            self.emit_nat32(value);
        }
        self
    }
}

/// # Local
impl IrEmitter {
    unsafe fn emit(&mut self, op_code: IrOpCode) {
        self.data.extend_from_slice(&(op_code as u16).to_le_bytes());
    }

    unsafe fn emit_register_id(&mut self, register_id: RegisterId) {
        self.data.push(mem::transmute(register_id));
    }

    unsafe fn emit_nat32(&mut self, value: u32) {
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
