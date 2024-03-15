use rokugo_backend_common::{FunctionId, ValueId};
use std::ops::Range;

#[derive(Debug)]
pub enum MirOpCode {
    // ! Memory
    /// # Layout
    /// - [`ValueId`] - Returned id of this value
    /// - [`i32`] - Literal value assigned to this value
    DefineInt32,

    // ! Control flow
    /// # Layout
    /// - [`ValueId`] - Id of value which is will be returned from this function
    ReturnValue,
    /// # Layout
    /// - [`ValueId`] - Id of value which is will be returned from called function
    /// - [`FunctionId`] - Id of called function
    /// - [`u8`] - Count of arguments passed to called function
    /// - [[`ValueId`]] - Arguments passed to called function
    Call,

    // ! Meta
    /// # Layout
    /// - [`Range<usize>`]
    MetaSpan,
}

#[derive(Debug, PartialEq)]
pub struct MirInstruction<'content> {
    pub data: MirInstructionData<'content>,
    pub meta: MirInstructionMeta,
}

#[derive(Debug, PartialEq)]
pub enum MirInstructionData<'content> {
    // ! Memory
    DefineInt32(ValueId, i32),
    // ! Control flow
    ReturnValue(ValueId),
    Call(ValueId, FunctionId, &'content [ValueId]),
}

#[derive(Debug, Default, PartialEq)]
pub struct MirInstructionMeta {
    pub span: Option<Range<usize>>,
}
