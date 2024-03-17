/// Container which holds the registers which are most optimal to chill in the current context.
/// # Remarks
/// This is a hint to the JIT compiler to chill one of these registers if all of them are busy, by previous
/// instructions.
#[derive(Default)]
pub struct RegisterChill {
    _default: (),
}

impl RegisterChill {
    pub fn to_le_bytes(self) -> [u8; 0] {
        []
    }
}
