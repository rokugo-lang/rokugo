use std::fmt::Display;

use smallvec::SmallVec;

pub mod built_in;
pub mod collection;

#[derive(Debug)]
pub struct Type {
    kind: TypeKind,
}

#[derive(Debug)]
pub enum TypeKind {
    BuiltIn(built_in::BuiltInType),
    Regular(RegularType),
    GenericConstructed(GenericConstructedType),
}

#[derive(Debug)]
pub struct RegularType {
    fields: SmallVec<[UnstableTypeId; 4]>,
}

#[derive(Debug)]
pub struct GenericConstructedType {
    _parent_type_id: UnstableTypeId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnstableTypeId(u64);

impl UnstableTypeId {
    pub const VOID: UnstableTypeId = UnstableTypeId(0);

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
