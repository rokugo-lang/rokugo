mod precedence;

use rokugo_ast::TreeKind;
use rokugo_diagnostic::{Importance, Severity};
use rokugo_lexis::{
    token::{Token, TokenKind},
    token_set::TokenSet,
};

use crate::{Closed, Parser};

use self::precedence::{tighter, Tighter};

fn precedence_parse(p: &mut Parser, left: &Token) {
    let mut lhs = prefix(p);

    loop {
        let right = p.peek();
        match tighter(p, left, &right) {
            Some(tighter @ (Tighter::Right | Tighter::Ambiguous)) => {
                let o = p.open_before(lhs);
                let kind = infix(p, &right);
                lhs = p.close(o, kind);
                if tighter == Tighter::Ambiguous {
                    p.emit(
                        Severity::Error
                            .diagnostic(format!(
                                "operator precedence between `{}` and `{}` is ambiguous",
                                p.text(left),
                                p.text(&right),
                            ))
                            .with_label(Importance::Primary.label(p.span(&left.range), ""))
                            .with_label(Importance::Primary.label(p.span(&right.range), ""))
                            .with_child(Severity::Help.diagnostic(
                                "try adding parentheses around one of these operators' expressions to disambiguate",
                                // TODO: show the expressions
                            )),
                    )
                }
            }
            Some(Tighter::Left) | None => break,
        }
    }
}

pub const PREFIXES: TokenSet = TokenSet::of(&[
    TokenKind::Integer,
    TokenKind::Decimal,
    TokenKind::Character,
    TokenKind::String,
    TokenKind::Tag,
    TokenKind::Identifier,
    TokenKind::Operator,
    TokenKind::LParen,
]);

fn prefix(p: &mut Parser) -> Closed {
    let token = p.peek();
    match token.kind {
        TokenKind::Integer
        | TokenKind::Decimal
        | TokenKind::Character
        | TokenKind::String
        | TokenKind::Tag => prefix_literal(p),
        TokenKind::Identifier | TokenKind::Operator => prefix_identifier(p),
        TokenKind::LParen => prefix_paren(p, &token),
        other => {
            assert!(!PREFIXES.includes(token.kind));

            // NOTE: Error tokens are emitted when the lexer cannot deal with the input, along with
            // a diagnostic at an appropriate position.
            // For this reason we do not want to emit a less precise diagnostic over the more
            // precise one the lexer emitted along the Error token.
            if other != TokenKind::Error {
                let span = p.current_span();
                p.emit(
                    Severity::Error
                        .diagnostic("expression expected")
                        .with_label(
                            Importance::Primary
                                .label(span, "this token does not start an expression"),
                        ),
                );
            }
            p.advance_with_error()
        }
    }
}

fn prefix_literal(p: &mut Parser) -> Closed {
    let o = p.open();
    p.advance();
    p.close(o, TreeKind::Literal)
}

fn prefix_identifier(p: &mut Parser) -> Closed {
    let o = p.open();
    p.advance();
    p.close(o, TreeKind::Identifier)
}

fn prefix_paren(p: &mut Parser, token: &Token) -> Closed {
    let o = p.open();
    p.advance();
    expression(p);
    p.expect(TokenKind::RParen, |p, span| {
        Severity::Error
            .diagnostic("expected `)` after expression to close parentheses `()`")
            .with_label(Importance::Primary.label(span, "this token was expected to be `)`"))
            .with_label(Importance::Primary.label(
                p.span(&token.range),
                "this `(` does not have a matching `)`",
            ))
    });
    p.close(o, TreeKind::Paren)
}

fn infix(p: &mut Parser, op: &Token) -> TreeKind {
    match op.kind {
        TokenKind::And
        | TokenKind::Or
        | TokenKind::Dot
        | TokenKind::Equals
        | TokenKind::Colon
        | TokenKind::Pipe
        | TokenKind::Ampersand
        | TokenKind::Arrow
        | TokenKind::Operator => infix_binary(p, op),
        _ if is_at_application(p) => infix_apply(p, op),
        _ => panic!("unhandled infix operator: {op:?} = {:?}", p.text(op)),
    }
}

fn infix_binary(p: &mut Parser, op: &Token) -> TreeKind {
    p.advance();
    precedence_parse(p, op);
    TreeKind::Binary
}

fn is_at_application(p: &Parser) -> bool {
    let current = p.peek();
    // Operators may not be used as application arguments, because it makes the case of pipelining
    // `x |> f |> g` impossible. It's also really quite unintuitive, given the visual separation
    // that operators give you.
    let can_be_an_argument = PREFIXES.includes(current.kind) && current.kind != TokenKind::Operator;
    // Application may not be separated by newlines. If you have that many arguments, use a record.
    let is_after_newline = p
        .preceding_trivia()
        .iter()
        .any(|t| t.kind != TokenKind::Newline);

    can_be_an_argument && !is_after_newline
}

fn infix_apply(p: &mut Parser, op: &Token) -> TreeKind {
    while is_at_application(p) {
        precedence_parse(p, op);
    }
    TreeKind::Apply
}

pub fn expression(p: &mut Parser) {
    precedence_parse(p, &p.eof_token())
}
