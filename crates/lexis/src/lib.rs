use lexer::Lexer;
use rokugo_diagnostic::Diagnostic;
use rokugo_source_code::{FileId, Sources};
use token::Token;

mod lexer;
pub mod token;

pub fn lex(sources: &Sources, file_id: FileId) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer {
        file_id,
        input: &sources.get(file_id).source,
        position: 0,
        tokens: vec![],
        diagnostics: vec![],
    };
    lexer.lex();
    (lexer.tokens, lexer.diagnostics)
}
