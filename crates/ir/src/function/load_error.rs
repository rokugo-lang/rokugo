use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use rokugo_diagnostic::{Diagnostic, Severity};

use crate::archive::{load_error::ArchiveLoadError, ArchiveRef};

use super::UnstableFunctionId;

#[derive(Debug)]
pub enum FunctionLoadError {
    DoesNotExsist(ArchiveRef, UnstableFunctionId),
    ArchiveRelated(ArchiveLoadError),
}

impl From<&FunctionLoadError> for Diagnostic {
    fn from(value: &FunctionLoadError) -> Self {
        match value {
            FunctionLoadError::DoesNotExsist(archive, function_id) => {
                Severity::Bug.diagnostic(format!(
                    "function with unstable id `{}` does not exist in archive `{}`",
                    function_id, archive
                ))
            }
            FunctionLoadError::ArchiveRelated(archive) => archive.into(),
        }
    }
}

impl Error for FunctionLoadError {}
impl Display for FunctionLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Diagnostic::from(self))
    }
}

impl From<ArchiveLoadError> for FunctionLoadError {
    fn from(value: ArchiveLoadError) -> Self {
        Self::ArchiveRelated(value)
    }
}
