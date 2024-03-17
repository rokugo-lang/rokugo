use rokugo_lexis::token::{Token, TokenKind};

/// Data structure which optimizes skipping over code comments and other "trivia" that can be
/// normally skipped by the parser, and are only useful when resolving certain specific cases.
///
/// More precisely, this accelerates skipping over comments and other whitespace tokens that do not
/// affect the AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSkipList {
    pub tokens: Vec<Token>,
    pub code: Vec<usize>,
}

impl TokenSkipList {
    /// Construct a new token skip list out of the given tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            code: tokens
                .iter()
                .enumerate()
                .filter_map(|(index, token)| Self::is_code_token(token.kind).then_some(index))
                .collect(),
            tokens,
        }
    }

    fn is_code_token(kind: TokenKind) -> bool {
        !matches!(kind, TokenKind::Newline | TokenKind::Comment)
    }

    /// Returns the code token at the given index.
    pub fn get(&self, position: usize) -> Option<&Token> {
        self.code.get(position).and_then(|&i| self.tokens.get(i))
    }

    /// Returns a pair of slices containing leading and trailing trivia of the token at the given
    /// index.
    ///
    /// Note that most tokens will have the right array empty. This is because the trailing trivia
    /// of token `n` could also be considered the leading trivia of token `n + 1`. We don't want
    /// this API to return the same tokens twice for more easy traversal.
    ///
    /// With this constraint, traversing the entire set of tokens, including trivia, looks
    /// like this:
    /// ```rust
    /// # use rokugo_parser::TokenSkipList;
    /// # let skip_list = TokenSkipList::new(vec![]);
    /// # fn do_stuff_with(tokens: &[rokugo_lexis::token::Token]) {}
    /// for &index in &skip_list.code {
    ///     let token = &skip_list.tokens[index];
    ///     let (leading, trailing) = skip_list.trivia(index);
    ///     do_stuff_with(leading);
    ///     do_stuff_with(std::slice::from_ref(token));
    ///     do_stuff_with(trailing);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// If `position` is out of bounds.
    pub fn trivia(&self, position: usize) -> (&[Token], &[Token]) {
        let pivot = self.code[position];
        if position == 0 {
            return (&self.tokens[..pivot], &[]);
        }

        let previous_pivot = self.code.get(position - 1).copied().unwrap_or(0);
        if position == self.code.len() - 1 {
            (
                &self.tokens[previous_pivot + 1..pivot],
                &self.tokens[pivot + 1..],
            )
        } else {
            (&self.tokens[previous_pivot + 1..pivot], &[])
        }
    }

    /// Returns the list of trivia occurring after the current code token (not including the token
    /// itself).
    pub fn before(&self, position: usize) -> &[Token] {
        if position >= self.code.len() {
            return &[];
        }

        let pivot = self.code[position];
        if position == 0 {
            &self.tokens[..pivot]
        } else {
            let previous_pivot = self.code[position - 1];
            &self.tokens[previous_pivot + 1..pivot]
        }
    }
}

#[cfg(test)]
mod tests {
    use rokugo_lexis::token::TokenKind;

    use super::TokenSkipList;

    #[test]
    fn skip_list() {
        let tokens = vec![
            TokenKind::Comment.at(0..5),
            TokenKind::Newline.at(5..6),
            TokenKind::Identifier.at(6..10),
        ];
        let skip_list = TokenSkipList::new(tokens.clone());
        assert_eq!(
            skip_list,
            TokenSkipList {
                tokens,
                code: vec![2]
            }
        );
        assert_eq!(skip_list.get(0), Some(&TokenKind::Identifier.at(6..10)))
    }

    #[test]
    fn trivia() {
        let tokens = vec![
            TokenKind::Comment.at(0..5),
            TokenKind::Newline.at(5..6),
            TokenKind::Identifier.at(6..10),
            TokenKind::Identifier.at(11..13),
            TokenKind::Comment.at(15..20),
        ];
        let skip_list = TokenSkipList::new(tokens.clone());

        assert_eq!(
            skip_list,
            TokenSkipList {
                tokens,
                code: vec![2, 3]
            }
        );

        assert_eq!(
            skip_list.trivia(0),
            (
                [TokenKind::Comment.at(0..5), TokenKind::Newline.at(5..6)].as_slice(),
                [].as_slice()
            )
        );
        assert_eq!(
            skip_list.trivia(1),
            ([].as_slice(), [TokenKind::Comment.at(15..20)].as_slice())
        );
    }
}
