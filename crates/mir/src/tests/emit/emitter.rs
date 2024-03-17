use crate::emit::{
    emitter::MirEmitter,
    op_code::{MirInstructionData, MirInstructionMeta},
};

fn emit_and_assert<const LENGTH: usize>(
    f: fn(&mut MirEmitter) -> [MirInstructionData<'static>; LENGTH],
) {
    let mut mir = MirEmitter::new();
    let data = f(&mut mir);
    for (i, instruction) in mir.iter().enumerate() {
        assert_eq!(data[i], instruction.data);
    }
}

fn emit_meta_and_assert(f: fn(&mut MirEmitter) -> MirInstructionMeta) {
    let mut mir = MirEmitter::new();
    let data = f(&mut mir);
    assert_eq!(data, mir.iter().next().unwrap().meta);
}

// ! Memory
#[test]
fn define_nat32() {
    emit_and_assert(|mir| {
        let id = mir.define_nat32(45);
        [MirInstructionData::DefineNat32(id, 45)]
    });
}

#[test]
fn define_int32() {
    emit_and_assert(|mir| {
        let id = mir.define_int32(65);
        [MirInstructionData::DefineInt32(id, 65)]
    });
}

// ! Control flow
#[test]
fn return_value() {
    emit_and_assert(|mir| {
        let id = mir.define_int32(-5634);
        mir.return_value(id);
        [
            MirInstructionData::DefineInt32(id, -5634),
            MirInstructionData::ReturnValue(id),
        ]
    });
}

// ! Meta
#[test]
fn meta_span() {
    emit_meta_and_assert(|mir| {
        mir.meta_span(0..5).define_int32(-1);
        MirInstructionMeta {
            span: Some(0..5),
            ..Default::default()
        }
    });
}
