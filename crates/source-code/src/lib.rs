//! Source code storage and handling.

use std::ops::Range;

/// Loaded source file.
#[derive(Debug, Clone)]
pub struct File {
    pub filename: String,
    pub source: String,
}

/// Unique identifier used to look up files inside [`Sources`].
///
/// The representation of this identifier is unspecified and may change between compilations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(usize);

/// Set of source files indexable by [`FileId`]s.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    files: Vec<File>,
}

impl Sources {
    /// Add a source file to the set, returning its ID.
    pub fn add(&mut self, file: File) -> FileId {
        let id = FileId(self.files.len());
        self.files.push(file);
        id
    }

    /// Get a source file from the set.
    pub fn get(&self, id: FileId) -> &File {
        &self.files[id.0]
    }

    pub fn span(&self, span: &SourceSpan) -> &str {
        &self.get(span.file_id).source[span.span.clone()]
    }
}

/// Span of bytes inside of a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub file_id: FileId,
    pub span: Range<usize>,
}

impl FileId {
    /// Constructs a [`SourceSpan`] at the given span of bytes in this file.
    pub fn span(self, span: Range<usize>) -> SourceSpan {
        SourceSpan {
            file_id: self,
            span,
        }
    }
}
