use rokugo_backend_common::FunctionId;
use rokugo_mir::archive_builder::ArchiveBuilderRef;

pub fn new_archive_with_standard() -> ArchiveBuilderRef {
    ArchiveBuilderRef::new()
}

pub fn compile_and_run<TResult, TArgument>(
    _archive: &ArchiveBuilderRef,
    _function: FunctionId,
    _argument: TArgument,
) -> TResult {
    unimplemented!()
}
