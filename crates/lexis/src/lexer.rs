use std::ops::Range;

use rokugo_diagnostic::{note, Diagnostic, Importance, NoteKind, Severity};
use rokugo_source_code::{FileId, SourceSpan};

use crate::token::{Token, TokenKind};

/// Lexer state.
pub struct Lexer<'a> {
    pub file_id: FileId,
    pub input: &'a str,
    pub position: usize,
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn current(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self) {
        self.position += self.current().map(|c| c.len_utf8()).unwrap_or(0);
    }

    fn span(&self, span: Range<usize>) -> SourceSpan {
        SourceSpan {
            file_id: self.file_id,
            span,
        }
    }

    fn token(&self, start: usize, kind: TokenKind) -> Token {
        kind.at(start..self.position)
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ' | '\r' | '\t') = self.current() {
            self.advance();
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) {
        let start = self.position;
        self.advance();
        self.tokens.push(self.token(start, kind));
    }

    fn comment(&mut self) {
        let start = self.position;
        self.advance();
        while !matches!(self.current(), Some('\n') | None) {
            self.advance();
        }
        let end = self.position;
        self.tokens.push(TokenKind::Comment.at(start..end));
    }

    fn decimal_number_literal(&mut self) {
        let start = self.position;
        let mut kind = TokenKind::Integer;
        while let Some('0'..='9') = self.current() {
            self.advance();
        }
        let decimal_point_start = self.position;
        if let Some('.') = self.current() {
            kind = TokenKind::Decimal;
            self.advance();
            let decimal_point_end = self.position;
            if !matches!(self.current(), Some('0'..='9')) {
                self.diagnostics.push(
                    Severity::Error
                        .diagnostic("decimal point `.` must be followed by at least one digit")
                        .with_label(
                            Importance::Primary
                                .label(self.span(decimal_point_start..decimal_point_end), ""),
                        ),
                );
            }
            while let Some('0'..='9') = self.current() {
                self.advance();
            }
        }
        self.tokens.push(kind.at(start..self.position));
    }

    fn character_or_escape(&mut self) {
        match self.current() {
            Some('\\') => {
                self.advance();
                let escaped_start = self.position;
                let escaped = self.current();
                let escaped_end = self.position;
                self.advance();
                if escaped == Some('u') {
                    if self.current() == Some('{') {
                        self.advance();
                        let codepoint_start = self.position;
                        while let Some('0'..='9' | 'A'..='F' | 'a'..='f') = self.current() {
                            self.advance();
                        }
                        let codepoint_end = self.position;
                        if self.current() == Some('}') {
                            self.advance();
                        } else {
                            self.diagnostics.push(
                                Severity::Error.diagnostic("`}` expected to close `\\u` Unicode code point escape sequence").with_label(
                                    Importance::Primary
                                        .label(self.span(codepoint_start..codepoint_end), "`}` expected after this"),
                                )
                            );
                        }
                    } else {
                        self.diagnostics.push(
                            Severity::Error
                                .diagnostic(
                                    "`{` expected after `\\u` Unicode code point escape sequence",
                                )
                                .with_label(Importance::Primary.label(
                                    self.span(escaped_start..escaped_end),
                                    "`{` expected after this",
                                )).with_note(note(NoteKind::Note, "Unicode code point escape sequences take the form `\\u{xx}`, where xx is a sequence of hexadecimal digits specifying the code point")),
                        );
                    }
                }
            }
            Some(_) => self.advance(),
            None => (),
        }
    }

    fn character_literal(&mut self) {
        let start = self.position;
        self.advance();
        self.character_or_escape();
        if self.current() != Some('\'') {
            self.diagnostics.push(
                Severity::Error
                    .diagnostic("missing `'` after character literal")
                    .with_label(
                        Importance::Primary
                            .label(self.span(start..self.position), "missing `'` after this"),
                    ),
            );
            self.tokens.push(TokenKind::Error.at(start..self.position));
        } else {
            self.advance();
            self.tokens
                .push(TokenKind::Character.at(start..self.position));
        }
    }

    fn string_literal(&mut self) {
        let mut is_multiline = false;

        let start = self.position;
        self.advance();
        let after_quote = self.position;
        while self.current() != Some('"') {
            self.advance();
            if self.current() == Some('\n') {
                is_multiline = true;
            }
            if self.current().is_none() {
                self.diagnostics.push(
                    Severity::Error
                        .diagnostic("missing `\"` to close string literal")
                        .with_label(Importance::Primary.label(
                            self.span(start..after_quote),
                            "missing `\"` to close this literal",
                        )),
                );
                break;
            }
        }
        self.advance(); // skip "

        if is_multiline {
            self.diagnostics.push(
                Severity::Error
                    .diagnostic("string literals may not span multiple lines")
                    .with_label(Importance::Primary.label(
                        self.span(start..self.position),
                        "this literal spans multiple lines",
                    )),
            );
        }

        self.tokens.push(TokenKind::String.at(start..self.position));
    }

    fn is_identifier_start_char(c: Option<char>) -> bool {
        matches!(c, Some('a'..='z' | 'A'..='Z' | '_'))
    }

    fn is_identifier_char(c: Option<char>) -> bool {
        Self::is_identifier_start_char(c) || matches!(c, Some('0'..='9'))
    }

    fn identifier(&mut self) {
        let start = self.position;
        while Self::is_identifier_char(self.current()) {
            self.advance();
        }
        let end = self.position;

        let identifier = &self.input[start..end];
        let kind = match identifier {
            "_" => TokenKind::Underscore,
            "and" => TokenKind::And,
            "break" => TokenKind::Break,
            "default" => TokenKind::Default,
            "do" => TokenKind::Do,
            "effect" => TokenKind::Effect,
            "else" => TokenKind::Else,
            "fun" => TokenKind::Fun,
            "handle" => TokenKind::Handle,
            "if" => TokenKind::If,
            "interface" => TokenKind::Interface,
            "internal" => TokenKind::Internal,
            "is" => TokenKind::Is,
            "let" => TokenKind::Let,
            "match" => TokenKind::Match,
            "module" => TokenKind::Module,
            "mut" => TokenKind::Mut,
            "or" => TokenKind::Or,
            "set" => TokenKind::Set,
            "then" => TokenKind::Then,
            "use" => TokenKind::Use,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            "with" => TokenKind::With,
            _ => TokenKind::Identifier,
        };

        self.tokens.push(kind.at(start..end));
    }

    fn is_operator_char(c: Option<char>) -> bool {
        matches!(
            c,
            Some(
                '+' | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '<'
                    | '='
                    | '>'
                    | '!'
                    | '?'
                    | '^'
                    | '&'
                    | '|'
                    | '~'
                    | '$'
                    | '.'
                    | ':'
                    | '@'
            )
        )
    }

    fn operator(&mut self) {
        let start = self.position;
        while Self::is_operator_char(self.current()) {
            self.advance();
        }
        let end = self.position;

        let operator = &self.input[start..end];
        let kind = match operator {
            "." => TokenKind::Dot,
            ":" => TokenKind::Colon,
            "=" => TokenKind::Equals,
            "|" => TokenKind::Pipe,
            "&" => TokenKind::Ampersand,
            "->" => TokenKind::Arrow,
            "@" => TokenKind::At,
            _ => TokenKind::Operator,
        };
        if kind == TokenKind::Colon && Self::is_identifier_char(self.current()) {
            if !Self::is_identifier_start_char(self.current()) {
                self.diagnostics.push(
                    Severity::Error
                        .diagnostic("tag name cannot start with a number")
                        .with_label(Importance::Primary.label(self.span(end..end + 1), ""))
                        .with_note(note(NoteKind::Note, "tag names must be valid identifiers.\nidentifiers cannot start with digits because they could be confused with numbers")),
                )
            }
            while Self::is_identifier_char(self.current()) {
                self.advance();
            }
            let end = self.position;
            self.tokens.push(TokenKind::Tag.at(start..end));
        } else {
            self.tokens.push(kind.at(start..end));
        }
    }

    /// Lexis loop.
    ///
    /// This lexer pushes tokens out to a [`Vec<Token>`], which can later be read via
    /// [`Lexer::tokens`]. It may also emit diagnostics while lexing, and these will be visible in
    /// [`Lexer::diagnostics`].
    pub fn lex(&mut self) {
        loop {
            self.skip_whitespace();

            if let Some(c) = self.current() {
                match c {
                    '\n' => self.single_char_token(TokenKind::Newline),
                    '#' => self.comment(),

                    '(' => self.single_char_token(TokenKind::LParen),
                    ')' => self.single_char_token(TokenKind::RParen),
                    '[' => self.single_char_token(TokenKind::LBracket),
                    ']' => self.single_char_token(TokenKind::RBracket),
                    '{' => self.single_char_token(TokenKind::LBrace),
                    '}' => self.single_char_token(TokenKind::RBrace),
                    ',' => self.single_char_token(TokenKind::Comma),
                    ';' => self.single_char_token(TokenKind::Semicolon),

                    '0'..='9' => self.decimal_number_literal(),
                    '\'' => self.character_literal(),
                    '"' => self.string_literal(),
                    _ if Self::is_identifier_start_char(Some(c)) => self.identifier(),
                    _ if Self::is_operator_char(Some(c)) => self.operator(),

                    _ => {
                        let start = self.position;
                        self.advance();
                        let span = start..self.position;
                        self.diagnostics.push(
                            Severity::Error
                                .diagnostic(format!("unexpected `{}`", c))
                                .with_label(Importance::Primary.label(
                                    self.span(span.clone()),
                                    "this character is not valid in Rokugo source code",
                                )),
                        );
                        self.tokens.push(self.token(start, TokenKind::Error));
                    }
                }
            } else {
                break;
            }
        }
    }
}
