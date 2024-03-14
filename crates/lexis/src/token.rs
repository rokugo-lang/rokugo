use std::ops::Range;

/// Kind of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Error,

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
}

/// A single token, representing a specific lexical construct at a given source span.
///
/// [`Token`]s do not store any information about the contents of the source span. The lexer will
/// only read characters and categorise them into tokens without trying to parse them into usable
/// data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

impl TokenKind {
    /// Construct a token of this kind, at the given span.
    pub fn at(self, span: Range<usize>) -> Token {
        Token { kind: self, span }
    }
}
