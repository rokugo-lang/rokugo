use crate::emit::{
    content::MirContent,
    emitter::MirEmitter,
    op_code::{MirInstruction, MirInstructionData, MirInstructionMeta},
};

#[test]
fn many() {
    let mut mir = MirEmitter::new();

    // Prepare
    let int = mir.meta_span(0..3).define_int32(65);
    mir.return_value(int);

    let content = MirContent::from(mir);
    let mut iter = content.iter();

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
