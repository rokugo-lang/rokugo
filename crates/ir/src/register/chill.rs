#![doc = include_str!("../../../../docs/ir/register/chilling.md")]

/// Container which holds the registers which are most optimal to chill in the current context.
///
/// # Remarks
///
/// This is a hint to the JIT compiler to chill one of these registers if all of them are busy, by previous
/// instructions.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegisterChill {}

impl RegisterChill {
    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 0);
        Self::default()
    }

    pub fn to_le_bytes(self) -> [u8; 0] {
        []
    }
}
