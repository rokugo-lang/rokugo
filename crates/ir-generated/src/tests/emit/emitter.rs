use rokugo_ir::{
    container::IrContainer,
    op_code::IrInstruction,
    register::{chill::RegisterChill, Register},
};

use crate::emit::emitter::IrEmitter;

#[test]
fn many() {
    let mut ir = IrEmitter::new();

    // Prepare
    let register = ir.alloc_register_nat32(RegisterChill::default()).unwrap();
    let register_id = register.id();
    ir.load_nat32(&register, 65);

    let container = IrContainer::from(ir);
    let mut iter = container.iter();

    // Assert
    assert!(Some(IrInstruction::LoadNat32(register_id, 65)) == iter.next());

    assert!(iter.next().is_none());
}
