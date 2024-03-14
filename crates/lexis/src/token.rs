use std::ops::Range;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

impl TokenKind {
    pub fn at(self, span: Range<usize>) -> Token {
        Token { kind: self, span }
    }
}
