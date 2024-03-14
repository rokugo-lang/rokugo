pub(crate) mod files;
mod render;

use rokugo_source_code::SourceSpan;

pub use render::render;
pub use render::Output;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Bug,
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Importance {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub importance: Importance,
    pub source_span: SourceSpan,
    pub message: String,
}

impl Importance {
    pub fn label(self, source_span: SourceSpan, message: impl Into<String>) -> Label {
        Label {
            importance: self,
            source_span,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    /// Contextual information that may not be noteworthy. Has no prefix.
    Context,
    /// Something noteworthy. Has a more attention-grabbing `note:` prefix.
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub kind: NoteKind,
    pub message: String,
}

pub fn note(kind: NoteKind, message: impl Into<String>) -> Note {
    Note {
        kind,
        message: message.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<Note>,
    pub children: Vec<Diagnostic>,
}

impl Severity {
    pub fn diagnostic(self, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: self,
            message: message.into(),
            labels: vec![],
            notes: vec![],
            children: vec![],
        }
    }
}

impl Diagnostic {
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_child(mut self, child: Diagnostic) -> Self {
        self.children.push(child);
        self
    }
}
