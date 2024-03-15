use std::{mem, ops::Range};

use rokugo_backend_common::{FunctionId, ValueId};

use super::{container::MirContainer, op_code::MirOpCode};

#[derive(Debug)]
pub struct MirEmitter {
    next_value_id: u32,
    content: MirContainer,
}

impl MirEmitter {
    pub fn new() -> Self {
        Self {
            next_value_id: 0,
            content: MirContainer { data: Vec::new() },
        }
    }
}

/// # Memory
impl MirEmitter {
    /// Defines a value with assigned literal `value` which is represented by `value_id`.
    pub fn define_int32(&mut self, value: i32) -> ValueId {
        unsafe {
            self.emit(MirOpCode::DefineInt32);
            let value_id = self.next_value_id();
            self.emit_value_id(value_id);
            self.emit_i32(value);

            value_id
        }
    }
}

/// # Control flow
impl MirEmitter {
    /// Returns from this function with the value which is represented by `value_id`. Function return type must be the
    /// same as type of the value.
    pub fn return_value(&mut self, value_id: ValueId) {
        unsafe {
            self.emit(MirOpCode::ReturnValue);
            self.emit_value_id(value_id);
        }
    }

    /// Calls a function which is represented by `function_id` with the arguments which are represented by `arguments`.
    /// And returns the value which is returned from the called function.
    pub fn call(
        &mut self,
        function_id: FunctionId,
        arguments: impl IntoIterator<Item = ValueId>,
    ) -> ValueId {
        unsafe {
            self.emit(MirOpCode::Call);
            let value_id = self.next_value_id();
            self.emit_value_id(value_id);
            self.emit_function_id(function_id);

            self.emit_u8(0);
            let position = self.content.data.len();

            let mut count = 0;
            for argument in arguments {
                self.emit_value_id(argument);
                count += 1;
            }

            self.content.data[position] = count;

            value_id
        }
    }
}

/// # Meta
impl MirEmitter {
    /// Adds meta data to the next instruction, which is represented by `span` what is a range of bytes in the
    /// frontend's source code which generated that instruction. This is useful for debugging and error reporting.
    pub fn meta_span(&mut self, span: Range<usize>) -> &mut Self {
        unsafe {
            self.emit(MirOpCode::MetaSpan);
            self.emit_usize(span.start);
            self.emit_usize(span.end);
        }
        self
    }
}

/// Internal
impl MirEmitter {
    /// # Safety
    /// This function is unsafe because it returns a [`VariableId`] which does not have to be properly registered in
    /// scope, what can cause a compiler or runtime panic. The caller must ensure that the [`VariableId`] is properly.
    pub(crate) unsafe fn next_value_id(&mut self) -> ValueId {
        let variable_id = mem::transmute(self.next_value_id);
        self.next_value_id += 1;
        variable_id
    }

    /// # Safety
    /// This function is unsafe because it can cause a compiler or runtime panic if the `op_code` is not properly.
    /// The caller must ensure that the `op_code` have properly values.
    unsafe fn emit(&mut self, op_code: MirOpCode) {
        self.content.emit_native_bytes(op_code);
    }

    unsafe fn emit_function_id(&mut self, function_id: FunctionId) {
        self.content.emit_native_bytes(function_id);
    }

    unsafe fn emit_value_id(&mut self, value_id: ValueId) {
        self.content.emit_native_bytes(value_id);
    }

    unsafe fn emit_u8(&mut self, u8: u8) {
        self.content.emit_native_bytes(u8);
    }

    unsafe fn emit_usize(&mut self, usize: usize) {
        self.content.emit_native_bytes(usize);
    }

    unsafe fn emit_i32(&mut self, i32: i32) {
        self.content.emit_native_bytes(i32);
    }
}

impl Default for MirEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MirEmitter> for MirContainer {
    fn from(mir_emitter: MirEmitter) -> Self {
        mir_emitter.content
    }
}
