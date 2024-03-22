use std::fmt::{self, Debug};

use indoc::indoc;
use rokugo_ast::Tree;
use rokugo_diagnostic::{Diagnostic, Output};
use rokugo_lexis::lex;
use rokugo_parser::{expression::expression, Parser, TokenSkipList};
use rokugo_source_code::{File, Sources};

struct ParseFailed {
    sources: Sources,
    diagnostics: Vec<Diagnostic>,
}

impl Debug for ParseFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered =
            rokugo_diagnostic::render(Output::Plain, &self.sources, self.diagnostics.clone());
        f.write_str(&String::from_utf8_lossy(&rendered))
    }
}

fn parse(production: fn(&mut Parser), filename: &str, source: &str) -> Result<Tree, ParseFailed> {
    let mut sources = Sources::default();
    let file_id = sources.add(File {
        filename: filename.to_owned(),
        source: source.to_owned(),
    });

    let (tokens, diagnostics) = lex(&sources, file_id);
    if !diagnostics.is_empty() {
        return Err(ParseFailed {
            sources,
            diagnostics,
        });
    }

    let mut parser = Parser::new(&sources, file_id, TokenSkipList::new(tokens));
    production(&mut parser);
    if !parser.diagnostics.is_empty() {
        Err(ParseFailed {
            diagnostics: parser.diagnostics,
            sources,
        })
    } else {
        Ok(parser.into_tree())
    }
}

fn expect_tree(
    production: fn(&mut Parser),
    filename: &str,
    source: &str,
    tree: &str,
) -> Result<(), ParseFailed> {
    let parsed = parse(production, filename, source)?.test_repr();
    let expected = tree;
    assert!(
        parsed == expected,
        "parsed tree did not meet expectations in {filename}\n\nparsed: {parsed}\n\nexpected: {expected}"
    );
    Ok(())
}

fn expect_error(production: fn(&mut Parser), filename: &str, source: &str) {
    if let Ok(tree) = parse(production, filename, source) {
        panic!("error expected, but got a valid tree: {tree:?}")
    }
}

#[test]
fn two_plus_two() -> Result<(), ParseFailed> {
    expect_tree(
        expression,
        "two_plus_two#1",
        "2 + 2",
        indoc! {"Binary {
            Literal {
                Integer @ 0..1
            }
            Operator @ 2..3
            Literal {
                Integer @ 4..5
            }
        }"},
    )?;
    expect_tree(
        expression,
        "two_plus_two#2",
        "2 + 2 + 2",
        indoc! {"Binary {
            Binary {
                Literal {
                    Integer @ 0..1
                }
                Operator @ 2..3
                Literal {
                    Integer @ 4..5
                }
            }
            Operator @ 6..7
            Literal {
                Integer @ 8..9
            }
        }"},
    )?;
    Ok(())
}

#[test]
fn chain() -> Result<(), ParseFailed> {
    expect_tree(
        expression,
        "chain#1",
        "2 + 2 + 2",
        indoc! {"Binary {
            Binary {
                Literal {
                    Integer @ 0..1
                }
                Operator @ 2..3
                Literal {
                    Integer @ 4..5
                }
            }
            Operator @ 6..7
            Literal {
                Integer @ 8..9
            }
        }"},
    )?;
    Ok(())
}

#[test]
fn math_precedence() -> Result<(), ParseFailed> {
    expect_tree(
        expression,
        "math_precedence#1",
        "2 * 2 + 2",
        indoc! {"Binary {
            Binary {
                Literal {
                    Integer @ 0..1
                }
                Operator @ 2..3
                Literal {
                    Integer @ 4..5
                }
            }
            Operator @ 6..7
            Literal {
                Integer @ 8..9
            }
        }"},
    )?;
    expect_tree(
        expression,
        "math_precedence#2",
        "2 + 2 * 2",
        indoc! {"Binary {
            Literal {
                Integer @ 0..1
            }
            Operator @ 2..3
            Binary {
                Literal {
                    Integer @ 4..5
                }
                Operator @ 6..7
                Literal {
                    Integer @ 8..9
                }
            }
        }"},
    )?;
    expect_tree(
        expression,
        "math_precedence#3 transitive property of precedence",
        "2 * 2 >= 4",
        indoc! {"Binary {
            Binary {
                Literal {
                    Integer @ 0..1
                }
                Operator @ 2..3
                Literal {
                    Integer @ 4..5
                }
            }
            Operator @ 6..8
            Literal {
                Integer @ 9..10
            }
        }"},
    )?;
    Ok(())
}

#[test]
fn ambiguous_precedence() -> Result<(), ParseFailed> {
    expect_error(expression, "ambiguous_precedence#1", "1 and 2 or 3");
    expect_error(expression, "ambiguous_precedence#2", "1 == 0 +- 0.0001");
    expect_error(expression, "ambiguous_precedence#3", "1 |> 2 >>= 3");
    Ok(())
}

#[test]
fn magic_precedence() -> Result<(), ParseFailed> {
    expect_tree(
        expression,
        "magic_precedence#1 dot",
        "a.b |> b.c",
        indoc! {"Binary {
            Binary {
                Identifier {
                    Identifier @ 0..1
                }
                Dot @ 1..2
                Identifier {
                    Identifier @ 2..3
                }
            }
            Operator @ 4..6
            Binary {
                Identifier {
                    Identifier @ 7..8
                }
                Dot @ 8..9
                Identifier {
                    Identifier @ 9..10
                }
            }
        }"},
    )?;
    expect_tree(
        expression,
        "magic_precedence#2 equals",
        "a = b + 1234",
        indoc! {"Binary {
            Identifier {
                Identifier @ 0..1
            }
            Equals @ 2..3
            Binary {
                Identifier {
                    Identifier @ 4..5
                }
                Operator @ 6..7
                Literal {
                    Integer @ 8..12
                }
            }
        }"},
    )?;
    expect_tree(
        expression,
        "magic_precedence#3 colon",
        "(a + 1) : Nat32",
        indoc! {"Binary {
            Paren {
                LParen @ 0..1
                Binary {
                    Identifier {
                        Identifier @ 1..2
                    }
                    Operator @ 3..4
                    Literal {
                        Integer @ 5..6
                    }
                }
                RParen @ 6..7
            }
            Colon @ 8..9
            Identifier {
                Identifier @ 10..15
            }
        }"},
    )?;
    expect_error(expression, "magic_precedence#4 colon", "a + 1 : Nat32");
    expect_tree(
        expression,
        "magic_precedence#5 pipe",
        ":a | :b | :c",
        indoc! {"Binary {
            Binary {
                Literal {
                    Tag @ 0..2
                }
                Pipe @ 3..4
                Literal {
                    Tag @ 5..7
                }
            }
            Pipe @ 8..9
            Literal {
                Tag @ 10..12
            }
        }"},
    )?;
    expect_error(expression, "magic_precedence#6 pipe", "a + 1 | b + 2");
    expect_tree(
        expression,
        "magic_precedence#7 ampersand",
        "A & B & C",
        indoc! {"Binary {
            Binary {
                Identifier {
                    Identifier @ 0..1
                }
                Ampersand @ 2..3
                Identifier {
                    Identifier @ 4..5
                }
            }
            Ampersand @ 6..7
            Identifier {
                Identifier @ 8..9
            }
        }"},
    )?;
    expect_error(expression, "magic_precedence#8 ampersand", "A | B & C");
    expect_tree(
        expression,
        "magic_precedence#9 arrow",
        "Nat -> Int",
        indoc! {"Binary {
            Identifier {
                Identifier @ 0..3
            }
            Arrow @ 4..6
            Identifier {
                Identifier @ 7..10
            }
        }"},
    )?;
    expect_tree(
        expression,
        "magic_precedence#10 arrow",
        "a : Nat -> Int",
        indoc! {"Binary {
            Identifier {
                Identifier @ 0..1
            }
            Colon @ 2..3
            Binary {
                Identifier {
                    Identifier @ 4..7
                }
                Arrow @ 8..10
                Identifier {
                    Identifier @ 11..14
                }
            }
        }"},
    )?;
    expect_tree(
        expression,
        "magic_precedence#11 arrow",
        "Nat -> Nat -> Nat",
        indoc! {"Binary {
            Identifier {
                Identifier @ 0..3
            }
            Arrow @ 4..6
            Binary {
                Identifier {
                    Identifier @ 7..10
                }
                Arrow @ 11..13
                Identifier {
                    Identifier @ 14..17
                }
            }
        }"},
    )?;
    Ok(())
}

#[test]
fn paren() -> Result<(), ParseFailed> {
    expect_tree(
        expression,
        "paren#1",
        "(2 + 2) * 2",
        indoc! {"Binary {
            Paren {
                LParen @ 0..1
                Binary {
                    Literal {
                        Integer @ 1..2
                    }
                    Operator @ 3..4
                    Literal {
                        Integer @ 5..6
                    }
                }
                RParen @ 6..7
            }
            Operator @ 8..9
            Literal {
                Integer @ 10..11
            }
        }"},
    )?;
    expect_error(expression, "missing_paren", "(2 + 2 * 2");
    Ok(())
}

#[test]
fn apply() -> Result<(), ParseFailed> {
    expect_tree(
        expression,
        "apply#1",
        "f 1 2 3",
        indoc! {"Apply {
            Identifier {
                Identifier @ 0..1
            }
            Literal {
                Integer @ 2..3
            }
            Literal {
                Integer @ 4..5
            }
            Literal {
                Integer @ 6..7
            }
        }"},
    )?;
    expect_tree(
        expression,
        "apply#2",
        "f a.x",
        indoc! {"Apply {
            Identifier {
                Identifier @ 0..1
            }
            Binary {
                Identifier {
                    Identifier @ 2..3
                }
                Dot @ 3..4
                Identifier {
                    Identifier @ 4..5
                }
            }
        }"},
    )?;
    expect_tree(
        expression,
        "apply#3",
        "(f x) y",
        indoc! {"Apply {
            Paren {
                LParen @ 0..1
                Apply {
                    Identifier {
                        Identifier @ 1..2
                    }
                    Identifier {
                        Identifier @ 3..4
                    }
                }
                RParen @ 4..5
            }
            Identifier {
                Identifier @ 6..7
            }
        }"},
    )?;
    expect_tree(
        expression,
        "apply#4",
        "f (x y)",
        indoc! {"Apply {
            Identifier {
                Identifier @ 0..1
            }
            Paren {
                LParen @ 2..3
                Apply {
                    Identifier {
                        Identifier @ 3..4
                    }
                    Identifier {
                        Identifier @ 5..6
                    }
                }
                RParen @ 6..7
            }
        }"},
    )?;
    expect_tree(
        expression,
        "apply#5",
        "String.cat \"a\" \"b\"",
        indoc! {"Apply {
            Binary {
                Identifier {
                    Identifier @ 0..6
                }
                Dot @ 6..7
                Identifier {
                    Identifier @ 7..10
                }
            }
            Literal {
                String @ 11..14
            }
            Literal {
                String @ 15..18
            }
        }"},
    )?;
    expect_tree(
        expression,
        "apply#6",
        "-1",
        indoc! {"Apply {
            Identifier {
                Operator @ 0..1
            }
            Literal {
                Integer @ 1..2
            }
        }"},
    )?;
    expect_tree(
        expression,
        "apply#7",
        "f x * f y",
        indoc! {"Binary {
            Apply {
                Identifier {
                    Identifier @ 0..1
                }
                Identifier {
                    Identifier @ 2..3
                }
            }
            Operator @ 4..5
            Apply {
                Identifier {
                    Identifier @ 6..7
                }
                Identifier {
                    Identifier @ 8..9
                }
            }
        }"},
    )?;
    Ok(())
}
