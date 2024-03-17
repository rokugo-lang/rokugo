use super::{Register, RegisterId, SPECIAL_ID_RANGE};

const fn get_id(index: u16) -> RegisterId {
    let i = SPECIAL_ID_RANGE.start + index;
    if i >= SPECIAL_ID_RANGE.end {
        panic!("Register index out of range");
    }
    RegisterId(i)
}

/// Stack pointer
pub struct RegisterSP;

impl Register for RegisterSP {
    fn id(&self) -> RegisterId {
        const ID: RegisterId = get_id(0);
        ID
    }
}
