use crate::emit::{
    container::MirContainer,
    emitter::MirEmitter,
    op_code::{MirInstruction, MirInstructionData, MirInstructionMeta},
};

#[test]
fn many() {
    let mut mir = MirEmitter::new();

    // Prepare
    let int = mir.meta_span(0..3).define_int32(65);
    mir.return_value(int);

    let container = MirContainer::from(mir);
    let mut iter = container.iter();

    // Assert
    assert!(
        Some(MirInstruction {
            data: MirInstructionData::DefineInt32(int, 65),
            meta: MirInstructionMeta { span: Some(0..3) }
        }) == iter.next()
    );
    assert!(
        Some(MirInstruction {
            data: MirInstructionData::ReturnValue(int),
            meta: MirInstructionMeta::default()
        }) == iter.next()
    );

    assert!(iter.next().is_none());
}
