use rokugo_ir::{
    container::IrContainer,
    op_code::IrInstruction,
    register::{self, Register},
};

use crate::emit::emitter::IrEmitter;

#[test]
fn many() {
    let mut ir = IrEmitter::new();

    // Prepare
    ir.load_nat32(register::X3, 65);

    let container = IrContainer::from(ir);
    let mut iter = container.iter();

    // Assert
    assert!(Some(IrInstruction::LoadNat32(register::X3.id(), 65)) == iter.next());

    assert!(iter.next().is_none());
}
