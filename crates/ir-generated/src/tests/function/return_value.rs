use rokugo_ir::{
    function::{Function, FunctionSignature, ReturnData, ReturnDataContainer},
    r#type::UnstableTypeId,
};
use smallvec::SmallVec;

use crate::emit::emitter::IrEmitter;

#[test]
fn return_value() {
    let mut ir = IrEmitter::new();

    Function::new(
        FunctionSignature::new(ReturnData {
            return_type: UnstableTypeId::VOID,
            container: ReturnDataContainer::Registers(SmallVec::new()),
        }),
        ir.into(),
    );
}
