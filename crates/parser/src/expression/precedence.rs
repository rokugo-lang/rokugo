use std::cmp::Ordering;

use rokugo_lexis::token::{Token, TokenKind};

use crate::Parser;

use super::PREFIXES;

/// Built-in precedence categories.
///
/// Operators whose precedence is defined by a category may mix between each other, if a precedence
/// relation exists between them. Some categories do not define precedence relations between each
/// other, which produces an error message about ambiguous precedence.
///
/// If you add or remove any categories, do not forget to update the language design documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Category {
    /// The `.` operator.
    Dot,
    /// Function application. Does not use an operator character, but is characterized by a prefix
    /// token appearing after another prefix token, without a newline inbetween.
    Apply,
    /// Operators: `*`, `/`.
    Multiplication,
    /// Operators: `+`, `-`.
    Summation,
    /// Operators: `==`, `!=`, `<`, `>`, `<=`, `>=`.
    Relation,
    /// The `->` operator.
    Arrow,
    /// The `=` operator.
    Equals,
    /// The `:` operator.
    Colon,
    /// The `and` operator.
    And,
    /// The `or` operator.
    Or,
    /// The `|` operator.
    Pipe,
    /// The `&` operator.
    Ampersand,

    /// Last category; used as an array count for `RELATION_TABLE`.
    #[doc(hidden)]
    Last,
}

const CATEGORY_COUNT: usize = Category::Last as usize;
type CategoryRelationTable = [[Option<Ordering>; CATEGORY_COUNT]; CATEGORY_COUNT];

static CATEGORY_RELATION_TABLE: CategoryRelationTable = {
    // NOTE: This has to use a subset of Rust since `static` initialization is quite limited
    // (in the same ways as `const` initialization.)
    // Using a OnceCell would be an unnecessary performance penalty, since all information is known
    // during compilation.

    use Category::*;

    let mut t = [[None; CATEGORY_COUNT]; CATEGORY_COUNT];

    // Add base less-than relationships here.
    // Transitive and symmetric relationships will be filled in automatically.
    t[Summation as usize][Multiplication as usize] = Some(Ordering::Less);
    t[Relation as usize][Summation as usize] = Some(Ordering::Less);
    t[And as usize][Relation as usize] = Some(Ordering::Less);
    t[Or as usize][Relation as usize] = Some(Ordering::Less);
    t[Colon as usize][Arrow as usize] = Some(Ordering::Less);
    let mut a = 0;
    while a < CATEGORY_COUNT {
        // The following are true for each category.
        t[a][Apply as usize] = Some(Ordering::Less);
        t[a][Dot as usize] = Some(Ordering::Less);
        t[Equals as usize][a] = Some(Ordering::Less);
        a += 1;
    }
    // Apply vs Dot has to be disambiguated. Apply < Dot.
    t[Apply as usize][Dot as usize] = Some(Ordering::Less);
    t[Dot as usize][Apply as usize] = None;

    // Transitive closure.
    let mut a = 0;
    while a < CATEGORY_COUNT {
        let mut b = 0;
        while b < CATEGORY_COUNT {
            let mut c = 0;
            while c < CATEGORY_COUNT {
                // If a < b and b < c, then a < c.
                if matches!(t[a][b], Some(Ordering::Less))
                    && matches!(t[b][c], Some(Ordering::Less))
                {
                    t[a][c] = Some(Ordering::Less);
                }
                c += 1;
            }
            b += 1;
        }
        a += 1;
    }

    // Symmetric closure.
    let mut a = 0;
    while a < CATEGORY_COUNT {
        let mut b = 0;
        while b < CATEGORY_COUNT {
            if matches!(t[a][b], Some(Ordering::Less)) {
                t[b][a] = Some(Ordering::Greater);
            }
            b += 1;
        }
        a += 1;
    }

    // Note that this table does not contain information about equality.
    // This is because equality in Precedence::partial_cmp is handled using a different, catch-all
    // branch, which also handles equality for custom precedence categories.

    t
};

/// Precedence categories.
///
/// This is a syntactic feature somewhat unique to Rokugo, as most languages define precedence
/// between all operators. Instead, Rokugo only defines precedence between specific pairs of
/// operators, and not every pair is defined.
///
/// This forces some operators with normally unclear precedence to be parenthesized. Such as in
/// this example:
/// ```text
/// x == (0.0 +- 0.0001)
/// ```
/// The `a +- b` has to be parenthesized here, because `==`'s and `+-`'s precedence categories
/// ([`Math`][PrecedenceCategory::Math] and [`Other`][PrecedenceCategory::Other] respectively)
/// are defined to not have any precedence relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Precedence<'a> {
    Category(Category),
    /// Other categories. May not mix with any other category.
    Custom(&'a str),
}

impl<'a> PartialOrd for Precedence<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (_, _) if self == other => Some(Ordering::Equal),

            // Equals is an exception from the usual rules, because it can be mixed with and has
            // lesser precedence than custom operators.
            (Self::Category(Category::Equals), Self::Custom(_)) => Some(Ordering::Less),
            (Self::Custom(_), Self::Category(Category::Equals)) => Some(Ordering::Greater),

            // Apply is an exception from the usual rules, because it can be mixed with and has
            // greater precedence than custom operators.
            (Self::Custom(_), Self::Category(Category::Apply)) => Some(Ordering::Less),
            (Self::Category(Category::Apply), Self::Custom(_)) => Some(Ordering::Greater),

            // Dot is an exception from the usual rules, because it can be mixed with and has
            // greater precedence than custom operators.
            (Self::Custom(_), Self::Category(Category::Dot)) => Some(Ordering::Less),
            (Self::Category(Category::Dot), Self::Custom(_)) => Some(Ordering::Greater),

            (Self::Category(a), Self::Category(b)) => {
                CATEGORY_RELATION_TABLE[*a as usize][*b as usize]
            }

            _ => None,
        }
    }
}

fn precedence<'a>(p: &'a Parser, token: &Token) -> Option<Precedence<'a>> {
    let text = p.text(token);
    match token.kind {
        TokenKind::And => Some(Precedence::Category(Category::And)),
        TokenKind::Or => Some(Precedence::Category(Category::Or)),
        TokenKind::Dot => Some(Precedence::Category(Category::Dot)),
        TokenKind::Equals => Some(Precedence::Category(Category::Equals)),
        TokenKind::Colon => Some(Precedence::Category(Category::Colon)),
        TokenKind::Pipe => Some(Precedence::Category(Category::Pipe)),
        TokenKind::Ampersand => Some(Precedence::Category(Category::Ampersand)),
        TokenKind::Arrow => Some(Precedence::Category(Category::Arrow)),
        TokenKind::Operator => match text {
            "*" | "/" => Some(Precedence::Category(Category::Multiplication)),
            "+" | "-" => Some(Precedence::Category(Category::Summation)),
            "==" | "!=" | "<" | ">" | "<=" | ">=" => Some(Precedence::Category(Category::Relation)),

            _ => Some(Precedence::Custom(text)),
        },
        // NOTE: Order matters. PREFIXES also includes Operator to handle negation. We can't let it
        // override our manual TokenKind::Operator implementation.
        k if PREFIXES.includes(k) => Some(Precedence::Category(Category::Apply)),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
}

impl Associativity {
    fn of(operator: &str) -> Associativity {
        match operator {
            "->" => Associativity::Right,
            _ => Associativity::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tighter {
    Left,
    Right,
    Ambiguous,
}

pub fn tighter(p: &Parser, left: &Token, right: &Token) -> Option<Tighter> {
    let right_precedence = precedence(p, right)?;
    let Some(left_precedence) = precedence(p, left) else {
        assert_eq!(left.kind, TokenKind::EndOfFile);
        return Some(Tighter::Right);
    };

    match left_precedence.partial_cmp(&right_precedence) {
        Some(Ordering::Less) => Some(Tighter::Right),
        Some(Ordering::Equal) => Some(match Associativity::of(p.text(right)) {
            Associativity::Left => Tighter::Left,
            Associativity::Right => Tighter::Right,
        }),
        Some(Ordering::Greater) => Some(Tighter::Left),
        None => Some(Tighter::Ambiguous),
    }
}
