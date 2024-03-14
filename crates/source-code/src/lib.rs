use std::ops::Range;

#[derive(Debug, Clone)]
pub struct File {
    pub filename: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(usize);

#[derive(Debug, Clone, Default)]
pub struct Sources {
    files: Vec<File>,
}

impl Sources {
    pub fn add(&mut self, file: File) -> FileId {
        let id = FileId(self.files.len());
        self.files.push(file);
        id
    }

    pub fn get(&self, id: FileId) -> &File {
        &self.files[id.0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub file_id: FileId,
    pub span: Range<usize>,
}

impl FileId {
    pub fn span(self, span: Range<usize>) -> SourceSpan {
        SourceSpan {
            file_id: self,
            span,
        }
    }
}
