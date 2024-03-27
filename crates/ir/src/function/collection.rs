use crate::archive::{ArchiveRef, UnstableArchiveId};

use super::{load_error::FunctionLoadError, Function, UnstableFunctionId};

#[derive(Debug)]
pub struct FunctionCollection {
    ids: Vec<FunctionCategory>,
    local: Vec<Function>,
}

impl FunctionCollection {
    pub fn get(
        archive: &ArchiveRef,
        id: UnstableFunctionId,
    ) -> Result<&Function, FunctionLoadError> {
        let data = archive.functions();
        let category = match data.ids.get(id.inner() as usize) {
            Some(c) => c,
            None => return Err(FunctionLoadError::DoesNotExsist(archive.clone(), id)),
        };

        match category {
            FunctionCategory::Local(function_position) => {
                Ok(&data.local[*function_position as usize])
            }
            FunctionCategory::External(dependency_id, function_id) => {
                let dependency = archive.load_dependency(*dependency_id)?;
                Self::get(dependency, *function_id)
            }
        }
    }
}

#[derive(Debug)]
enum FunctionCategory {
    Local(u64),
    External(UnstableArchiveId, UnstableFunctionId),
}
