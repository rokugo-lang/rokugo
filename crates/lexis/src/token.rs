use std::{fmt, ops::Range};

use enum_iterator::Sequence;

/// Kind of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum TokenKind {
    Error,

    /// Not actually emitted by the lexer, but used by the parser to signal that it reached
    /// EOF without having to wrap the `TokenKind` in an [`Option<T>`].
    EndOfFile,

    // Metadata
    Comment,
    Newline,

    // Literals
    Integer,
    Decimal,
    Character,
    String,
    Tag,

    // Names
    Identifier,
    Operator,

    // Punctuators
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }
    Comma,     // ,
    Semicolon, // ;

    // Keywords
    And,
    Break,
    Default,
    Do,
    Effect,
    Else,
    Fun,
    Handle,
    If,
    Interface,
    Internal,
    Is,
    Let,
    Match,
    Module,
    Mut,
    Or,
    Set,
    Then,
    Underscore, // _
    Use,
    Var,
    While,
    With,

    // Reserved operators
    Dot,       // .
    Colon,     // :
    Equals,    // =
    Pipe,      // |
    Ampersand, // &
    At,        // @
    Arrow,     // ->
}

/// A single token, representing a specific lexical construct at a given source span.
///
/// [`Token`]s do not store any information about the contents of the source span. The lexer will
/// only read characters and categorise them into tokens without trying to parse them into usable
/// data.
#[derive(Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub range: Range<usize>,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {:?}", self.kind, self.range)
    }
}

impl TokenKind {
    /// Construct a token of this kind, at the given range of characters.
    pub fn at(self, range: Range<usize>) -> Token {
        Token { kind: self, range }
    }
}
