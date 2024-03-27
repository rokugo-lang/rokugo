use std::{fmt::Display, sync::Arc};

use crate::function::collection::FunctionCollection;

use self::load_error::ArchiveLoadError;

pub mod load_error;

#[derive(Debug, Clone)]
pub struct ArchiveRef {
    archive: Arc<Archive>,
}

impl ArchiveRef {
    pub fn load_dependency(&self, id: UnstableArchiveId) -> Result<&ArchiveRef, ArchiveLoadError> {
        match self.archive.dependencies.get(id.inner() as usize) {
            Some(a) => Ok(a),
            None => Err(ArchiveLoadError::DependencyDoesNotExist(self.clone(), id)),
        }
    }

    pub(crate) fn functions(&self) -> &FunctionCollection {
        &self.archive.functions
    }
}

impl Display for ArchiveRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: use more user friendly name
        write!(f, "{:?}", Arc::into_raw(self.archive.clone()))?;
        Ok(())
    }
}

#[derive(Debug)]
struct Archive {
    dependencies: Vec<ArchiveRef>,
    functions: FunctionCollection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnstableArchiveId(u64);

impl UnstableArchiveId {
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl Display for UnstableArchiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)?;
        Ok(())
    }
}
