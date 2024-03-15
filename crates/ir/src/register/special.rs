use super::{traits::memory::RegisterAddress, Register, RegisterId, S_START_INDEX};

pub struct RegisterSP;

impl Register for RegisterSP {
    fn id(&self) -> RegisterId {
        RegisterId(S_START_INDEX)
    }
}

impl RegisterAddress for RegisterSP {}
