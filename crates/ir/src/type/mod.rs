use std::fmt::Display;

use smallvec::SmallVec;

#[derive(Debug)]
pub enum Type {
    Linear(LinearType),
    GenericConstructed(GenericConstructedType),
}

#[derive(Debug)]
pub struct LinearType {
    fields: SmallVec<[UnstableTypeId; 4]>,
}

#[derive(Debug)]
pub struct GenericConstructedType {
    parent_type_id: UnstableTypeId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnstableTypeId(u64);

impl UnstableTypeId {
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl Display for UnstableTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)?;
        Ok(())
    }
}
