use rokugo_ir::{
    container::IrContainer,
    op_code::IrInstruction,
    register::{chill::RegisterChill, Register},
};

use crate::emit::emitter::IrEmitter;

fn emit_and_assert<const LENGTH: usize>(f: fn(&mut IrEmitter) -> [IrInstruction<'static>; LENGTH]) {
    let mut ir = IrEmitter::new();
    let data = f(&mut ir);

    let mut i = 0;
    for instruction in IrContainer::from(ir).iter() {
        assert_eq!(data[i], instruction.unwrap());
        i += 1;
    }

    assert_eq!(data.len() - 1, i);
}

#[test]
fn load_nat32() {
    emit_and_assert(|ir| {
        let chill = RegisterChill::default();
        let register = ir.alloc_register_nat32(chill.clone()).unwrap();
        let register_id = register.id();
        ir.load_nat32(&register, 65).drop_register(register);

        [
            IrInstruction::AllocRegisterNat32(register_id, chill),
            IrInstruction::LoadNat32(register_id, 65),
            IrInstruction::DropRegister(register_id),
        ]
    });
}
