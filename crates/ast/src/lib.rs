use std::fmt;

use rokugo_lexis::token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeKind {
    Error,

    // Which literal this is can be determined by looking at the token kind.
    Literal,
    Identifier,

    Paren,
    Binary,
    Apply,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Tree {
    pub kind: TreeKind,
    pub children: Vec<Child>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Child {
    Token(Token),
    Tree(Tree),
}

impl Tree {
    /// Returns a representation of the tree that's stable for testing.
    pub fn test_repr(&self) -> String {
        format!("{self:?}")
    }
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        test_repr::tree(f, self, 0)
    }
}

impl fmt::Debug for Child {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        test_repr::child(f, self, 0)
    }
}

/// Textual AST representation that is stable (and used for) unit and integration testing.
mod test_repr {
    use core::fmt;

    use rokugo_lexis::token::Token;

    use crate::{Child, Tree};

    fn indentation(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        for _ in 0..indent {
            f.write_str("    ")?;
        }
        Ok(())
    }

    fn token(f: &mut fmt::Formatter<'_>, token: &Token, indent: usize) -> fmt::Result {
        indentation(f, indent)?;
        write!(f, "{token:?}")
    }

    pub fn child(f: &mut fmt::Formatter<'_>, child: &Child, indent: usize) -> fmt::Result {
        match child {
            Child::Token(t) => token(f, t, indent),
            Child::Tree(t) => tree(f, t, indent),
        }
    }

    pub fn tree(f: &mut fmt::Formatter<'_>, tree: &Tree, indent: usize) -> fmt::Result {
        indentation(f, indent)?;
        write!(f, "{:?} {{", tree.kind)?;

        for c in &tree.children {
            f.write_str("\n")?;
            child(f, c, indent + 1)?;
        }

        if tree.children.is_empty() {
            f.write_str("}")?;
        } else {
            f.write_str("\n")?;
            indentation(f, indent)?;
            f.write_str("}")?;
        }
        Ok(())
    }
}
