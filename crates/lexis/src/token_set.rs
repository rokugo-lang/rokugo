use enum_iterator::Sequence;

use crate::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSet {
    bits: [u8; Self::SIZE],
}

impl TokenSet {
    const SIZE: usize = (TokenKind::CARDINALITY + 7) / 8;

    pub const fn empty() -> Self {
        Self {
            bits: [0; Self::SIZE],
        }
    }

    const fn byte_index(kind: TokenKind) -> usize {
        kind as usize / 8
    }

    const fn bit_index(kind: TokenKind) -> usize {
        kind as usize % 8
    }

    pub const fn of(kinds: &[TokenKind]) -> TokenSet {
        let mut token_set = TokenSet::empty();
        let mut i = 0;
        while i < kinds.len() {
            token_set.bits[Self::byte_index(kinds[i])] |= 1 << Self::bit_index(kinds[i]);
            i += 1;
        }
        token_set
    }

    pub const fn includes(&self, kind: TokenKind) -> bool {
        self.bits[Self::byte_index(kind)] & (1 << Self::bit_index(kind)) != 0
    }
}
