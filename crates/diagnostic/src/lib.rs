//! Rich, structured diagnostic message support, inspired by rustc.

pub(crate) mod files;
mod render;

use std::fmt;

use rokugo_source_code::SourceSpan;

pub use render::render;
pub use render::Output;
use rokugo_source_code::Sources;

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Compiler bug.
    Bug,
    /// Compilation error. Archives not emitted.
    Error,
    /// Compilation warning. Does not prevent running the program, but continuous integration suites
    /// should not allow for submissions to the source repository if warnings are present.
    Warning,

    /// Note attached to another diagnostic.
    Note,
    /// Piece of help attached to another diagnostic.
    Help,
}

/// Importance of a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Importance {
    /// Shows where the exact error is.
    Primary,
    /// Points to extra context information about the error.
    Secondary,
}

/// Labels are a way of attaching spans of source code to diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub importance: Importance,
    pub source_span: SourceSpan,
    /// Optional message; can be empty, and should not contain newlines to render properly.
    pub message: String,
}

impl Importance {
    /// Construct a label of this importance.
    pub fn label(self, source_span: SourceSpan, message: impl Into<String>) -> Label {
        Label {
            importance: self,
            source_span,
            message: message.into(),
        }
    }
}

/// The kind of a note attached to a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    /// Contextual information that may not be noteworthy. Has no prefix.
    Context,
    /// Something noteworthy. Has a more attention-grabbing `note:` prefix.
    Note,
}

/// Note attached to a diagnostic. Notes render below diagnostics and provide additional information
/// that is independent of source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub kind: NoteKind,
    pub message: String,
}

/// Construct a [`Note`] more conveniently.
pub fn note(kind: NoteKind, message: impl Into<String>) -> Note {
    Note {
        kind,
        message: message.into(),
    }
}

/// A structured diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity of this diagnostic.
    pub severity: Severity,
    /// Message attached to this diagnostic. This should be a short summary of what the diagnostic
    /// is about, containing key information for identifying the issue.
    ///
    /// The message is the only part of a diagnostic that is guaranteed to be shown.
    /// Any extra information may be omitted depending on what the environment allows.
    /// All other parts of the diagnostic should be written with that in mind.
    pub message: String,
    /// Labels attached to the diagnostic, identifying spans of source code the diagnostic
    /// should point to.
    pub labels: Vec<Label>,
    /// Notes attached to the diagnostic. These are not attached to any source code, but provide
    /// extra insights as to why the diagnostic was emitted, and what can be done to fix it.
    pub notes: Vec<Note>,
    /// Child diagnostics. These are emitted along with this diagnostic and should be considered
    /// extensions of what this diagnostic has to say.
    pub children: Vec<Diagnostic>,
}

impl Severity {
    /// Construct a diagnostic with this severity and a message.
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
    /// Add a label to this diagnostic.
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    /// Add a note to this diagnostic.
    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    /// Add a child to this diagnostic.
    pub fn with_child(mut self, child: Diagnostic) -> Self {
        self.children.push(child);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let render_bytes = render::render(
            render::Output::Plain,
            &Sources::default(),
            vec![self.clone()],
        );
        writeln!(f, "{}", String::from_utf8_lossy(&render_bytes))?;
        Ok(())
    }
}
