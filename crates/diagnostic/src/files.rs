use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Range,
};

use codespan_reporting::files::{self, line_starts, Files};
use rokugo_source_code::{FileId, Sources};

use crate::Diagnostic;

/// Sources preprocessed for emitting diagnostics.
pub struct DiagnosableSources<'a> {
    sources: &'a Sources,
    line_starts: HashMap<FileId, Vec<usize>>,
}

impl<'a> DiagnosableSources<'a> {
    /// Creates [`DiagnosableSources`] from a set of diagnostics.
    pub fn new(sources: &'a Sources, diagnostics: &[Diagnostic]) -> Self {
        let mut diagnosable_sources = Self {
            sources,
            line_starts: HashMap::new(),
        };
        for diagnostic in diagnostics {
            for label in &diagnostic.labels {
                diagnosable_sources.add_line_starts(label.source_span.file_id);
            }
        }
        diagnosable_sources
    }

    fn add_line_starts(&mut self, file_id: FileId) {
        if let Entry::Vacant(e) = self.line_starts.entry(file_id) {
            e.insert(line_starts(&self.sources.get(file_id).source).collect());
        }
    }

    fn line_start(&self, file_id: FileId, line_index: usize) -> Result<usize, files::Error> {
        use std::cmp::Ordering;

        let line_starts = &self.line_starts[&file_id];

        match line_index.cmp(&line_starts.len()) {
            Ordering::Less => Ok(line_starts
                .get(line_index)
                .cloned()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.sources.get(file_id).source.len()),
            Ordering::Greater => Err(files::Error::LineTooLarge {
                given: line_index,
                max: line_starts.len() - 1,
            }),
        }
    }
}

impl<'a> Files<'a> for DiagnosableSources<'a> {
    type FileId = FileId;

    type Name = &'a str;

    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, files::Error> {
        Ok(&self.sources.get(id).filename)
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, files::Error> {
        Ok(&self.sources.get(id).source)
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, files::Error> {
        Ok(self.line_starts[&id]
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1))
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<Range<usize>, files::Error> {
        let line_start = self.line_start(id, line_index)?;
        let next_line_start = self.line_start(id, line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}
