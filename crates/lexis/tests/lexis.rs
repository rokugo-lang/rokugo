use rokugo_diagnostic::{Diagnostic, Importance, Output, Severity};
use rokugo_lexis::token::{Token, TokenKind};
use rokugo_source_code::{File, FileId, Sources};

#[track_caller]
fn lex(filename: &str, source: &str) -> (Sources, FileId, Vec<Token>, Vec<Diagnostic>) {
    let mut sources = Sources::default();
    let file_id = sources.add(File {
        filename: filename.into(),
        source: source.into(),
    });
    let (tokens, diagnostics) = rokugo_lexis::lex(&sources, file_id);
    (sources, file_id, tokens, diagnostics)
}

// Nice tests are those that are 100% good for the compiler and do not emit any diagnostics.
#[track_caller]
fn nice(filename: &str, source: &str) -> Vec<Token> {
    let (sources, _file_id, tokens, diagnostics) = lex(filename, source);
    if !diagnostics.is_empty() {
        let rendered = rokugo_diagnostic::render(Output::Colored, &sources, diagnostics);
        let rendered = String::from_utf8_lossy(&rendered);
        panic!("test failure, diagnostics were emitted:\n{rendered}");
    }
    tokens
}

// Naughty tests may emit any sort of diagnostic in the process.
#[track_caller]
fn naughty(
    filename: &str,
    source: &str,
    expected_tokens: &[Token],
    expected_diagnostics: impl FnOnce(FileId) -> Vec<Diagnostic>,
) {
    let (_sources, file_id, tokens, diagnostics) = lex(filename, source);
    assert_eq!(tokens, expected_tokens);
    assert_eq!(diagnostics, expected_diagnostics(file_id));
}

#[test]
fn single_char_tokens() {
    assert_eq!(
        nice("single_char_tokens", "()[]{},;"),
        &[
            TokenKind::LParen.at(0..1),
            TokenKind::RParen.at(1..2),
            TokenKind::LBracket.at(2..3),
            TokenKind::RBracket.at(3..4),
            TokenKind::LBrace.at(4..5),
            TokenKind::RBrace.at(5..6),
            TokenKind::Comma.at(6..7),
            TokenKind::Semicolon.at(7..8),
        ]
    );
}

#[test]
fn invalid_characters() {
    naughty(
        "invalid_characters",
        "`",
        &[TokenKind::Error.at(0..1)],
        |file_id| {
            vec![Severity::Error.diagnostic("unexpected ```").with_label(
                Importance::Primary.label(
                    file_id.span(0..1),
                    "this character is not valid in Rokugo source code",
                ),
            )]
        },
    );
}

#[test]
fn whitespace() {
    assert_eq!(nice("just whitespace", "    \t    \r"), &[]);
    assert_eq!(
        nice("newline", "    \n    \n"),
        &[TokenKind::Newline.at(4..5), TokenKind::Newline.at(9..10)]
    );
}

#[test]
fn comment() {
    assert_eq!(nice("comment", "# abc"), &[TokenKind::Comment.at(0..5)]);
    assert_eq!(
        nice("comment followed by newline", "# abc\n"),
        &[TokenKind::Comment.at(0..5), TokenKind::Newline.at(5..6)]
    );
}

#[test]
fn integer() {
    assert_eq!(nice("integer 1", "1"), &[TokenKind::Integer.at(0..1)]);
    assert_eq!(nice("integer 2", "123"), &[TokenKind::Integer.at(0..3)]);
}

#[test]
fn decimal() {
    assert_eq!(nice("decimal 1", "1.0"), &[TokenKind::Decimal.at(0..3)]);
    assert_eq!(nice("decimal 2", "123.456"), &[TokenKind::Decimal.at(0..7)]);
}

#[test]
fn character() {
    assert_eq!(nice("character", "'a'"), &[TokenKind::Character.at(0..3)]);
    assert_eq!(
        nice("UTF-8 character", "'ł'"),
        &[TokenKind::Character.at(0..4)]
    );
}

#[test]
fn escapes() {
    assert_eq!(
        nice("escapes 1", "'\\\\'"),
        &[TokenKind::Character.at(0..4)]
    );
    assert_eq!(nice("escapes 2", "'\\''"), &[TokenKind::Character.at(0..4)]);
    assert_eq!(
        nice("escapes 3", "'\\\"'"),
        &[TokenKind::Character.at(0..4)]
    );
    assert_eq!(nice("escapes 4", "'\\n'"), &[TokenKind::Character.at(0..4)]);
    assert_eq!(nice("escapes 5", "'\\r'"), &[TokenKind::Character.at(0..4)]);
    assert_eq!(nice("escapes 6", "'\\t'"), &[TokenKind::Character.at(0..4)]);
    assert_eq!(
        nice("escapes 7", "'\\u{0A}'"),
        &[TokenKind::Character.at(0..8)]
    );
}

#[test]
fn string() {
    assert_eq!(
        nice("string", r#" "hello" "#),
        &[TokenKind::String.at(1..8)]
    );
    assert_eq!(
        nice("string escapes", r#" "hello\nworld" "#),
        &[TokenKind::String.at(1..15)]
    );
}

#[test]
fn identifier() {
    assert_eq!(
        nice("identifier lowercase", "lowercase"),
        &[TokenKind::Identifier.at(0..9)]
    );
    assert_eq!(
        nice("identifier camelCase", "camelCase"),
        &[TokenKind::Identifier.at(0..9)]
    );
    assert_eq!(
        nice("identifier snake_case", "snake_case"),
        &[TokenKind::Identifier.at(0..10)]
    );
    assert_eq!(
        nice("identifier PascalCase", "PascalCase"),
        &[TokenKind::Identifier.at(0..10)]
    );
    assert_eq!(
        nice("identifier digits", "a1234"),
        &[TokenKind::Identifier.at(0..5)]
    );
}

#[test]
fn operator() {
    assert_eq!(
        nice("arithmetic operators", "+ - * /"),
        &[
            TokenKind::Operator.at(0..1),
            TokenKind::Operator.at(2..3),
            TokenKind::Operator.at(4..5),
            TokenKind::Operator.at(6..7),
        ]
    );
    assert_eq!(
        nice("operator with all allowed characters", "+-*/%<=>!?^&|~$.:@"),
        &[TokenKind::Operator.at(0..18)]
    );
    assert_eq!(
        nice(
            "crazy operator",
            "++++++++++++++++++++++++!!!!!!!!!!!!!!!!!!!!!??????????????????"
        ),
        &[TokenKind::Operator.at(0..63)]
    );
}

#[test]
fn reserved_operators() {
    assert_eq!(
        nice("reserved operators", ". : = | & @"),
        &[
            TokenKind::Dot.at(0..1),
            TokenKind::Colon.at(2..3),
            TokenKind::Equals.at(4..5),
            TokenKind::Pipe.at(6..7),
            TokenKind::Ampersand.at(8..9),
            TokenKind::At.at(10..11),
        ]
    );
}

#[test]
fn tags() {
    assert_eq!(nice("tags", ":hug"), &[TokenKind::Tag.at(0..4)]);
}

#[test]
fn keywords() {
    let keywords = "_ and break default do effect else fun handle if interface internal is let match module mut or set then use var while with";
    assert_eq!(
        nice("keywords", keywords),
        &[
            TokenKind::Underscore.at(0..1),
            TokenKind::And.at(2..5),
            TokenKind::Break.at(6..11),
            TokenKind::Default.at(12..19),
            TokenKind::Do.at(20..22),
            TokenKind::Effect.at(23..29),
            TokenKind::Else.at(30..34),
            TokenKind::Fun.at(35..38),
            TokenKind::Handle.at(39..45),
            TokenKind::If.at(46..48),
            TokenKind::Interface.at(49..58),
            TokenKind::Internal.at(59..67),
            TokenKind::Is.at(68..70),
            TokenKind::Let.at(71..74),
            TokenKind::Match.at(75..80),
            TokenKind::Module.at(81..87),
            TokenKind::Mut.at(88..91),
            TokenKind::Or.at(92..94),
            TokenKind::Set.at(95..98),
            TokenKind::Then.at(99..103),
            TokenKind::Use.at(104..107),
            TokenKind::Var.at(108..111),
            TokenKind::While.at(112..117),
            TokenKind::With.at(118..122)
        ]
    );
}
