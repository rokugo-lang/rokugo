use codespan_reporting::term::{
    termcolor::{Ansi, ColorChoice, NoColor, StandardStream, WriteColor},
    Config,
};
use rokugo_source_code::Sources;
use tracing::error;

use crate::{files::DiagnosableSources, Diagnostic, Importance, NoteKind, Severity};

/// Kind of output that should be rendered.
///
/// Note that if stdout is incapable of rendering color, output will be set to [`Plain`][`Output::Plain`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    Plain,
    Colored,
}

fn render_rec(
    stream: &mut dyn WriteColor,
    sources: &DiagnosableSources,
    diagnostics: Vec<Diagnostic>,
) {
    for diagnostic in diagnostics {
        let codespan_diagnostic = codespan_reporting::diagnostic::Diagnostic {
            severity: match diagnostic.severity {
                Severity::Bug => codespan_reporting::diagnostic::Severity::Bug,
                Severity::Error => codespan_reporting::diagnostic::Severity::Error,
                Severity::Warning => codespan_reporting::diagnostic::Severity::Warning,
                Severity::Note => codespan_reporting::diagnostic::Severity::Note,
                Severity::Help => codespan_reporting::diagnostic::Severity::Help,
            },
            code: None,
            message: diagnostic.message,
            labels: diagnostic
                .labels
                .into_iter()
                .map(|label| codespan_reporting::diagnostic::Label {
                    style: match label.importance {
                        Importance::Primary => codespan_reporting::diagnostic::LabelStyle::Primary,
                        Importance::Secondary => {
                            codespan_reporting::diagnostic::LabelStyle::Secondary
                        }
                    },
                    file_id: label.source_span.file_id,
                    range: label.source_span.span,
                    message: label.message,
                })
                .collect(),
            notes: diagnostic
                .notes
                .into_iter()
                .map(|note| match note.kind {
                    NoteKind::Context => note.message,
                    NoteKind::Note => format!("note: {}", note.message),
                })
                .collect(),
        };
        match codespan_reporting::term::emit(
            stream,
            &Config::default(),
            sources,
            &codespan_diagnostic,
        ) {
            Ok(_) => (),
            Err(err) => error!(?codespan_diagnostic, ?err, "could not emit diagnostic"),
        }
        render_rec(stream, sources, diagnostic.children);
    }
}

/// Render diagnostics to a buffer of bytes.
/// This buffer of bytes can later be written out to stdout or a file.
pub fn render(mut output: Output, sources: &Sources, diagnostics: Vec<Diagnostic>) -> Vec<u8> {
    if !StandardStream::stdout(ColorChoice::Auto).supports_color() {
        output = Output::Plain;
    }

    let mut plain = NoColor::new(vec![]);
    let mut colored = Ansi::new(vec![]);
    let stream: &mut dyn WriteColor = match output {
        Output::Plain => &mut plain,
        Output::Colored => &mut colored,
    };

    let files = DiagnosableSources::new(sources, &diagnostics);
    render_rec(stream, &files, diagnostics);

    match output {
        Output::Plain => plain.into_inner(),
        Output::Colored => colored.into_inner(),
    }
}
