use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use rokugo_diagnostic::{Diagnostic, Severity};

use crate::archive::{ArchiveRef, UnstableArchiveId};

#[derive(Debug)]
pub enum ArchiveLoadError {
    DependencyDoesNotExist(ArchiveRef, UnstableArchiveId),
}

impl From<&ArchiveLoadError> for Diagnostic {
    fn from(value: &ArchiveLoadError) -> Self {
        match value {
            ArchiveLoadError::DependencyDoesNotExist(archive, dependency_id) => Severity::Bug
                .diagnostic(format!(
                    "dependency archive with unstable id `{}` does not exist for archive `{}`",
                    dependency_id, archive
                )),
        }
    }
}

impl Error for ArchiveLoadError {}
impl Display for ArchiveLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Diagnostic::from(self))
    }
}
