use crate::archive::UnstableArchiveId;

use super::{Type, UnstableTypeId};

#[derive(Debug)]
pub struct TypeCollection {
    ids: Vec<TypeCategory>,
    local: Vec<Type>,
}

#[derive(Debug)]
enum TypeCategory {
    Local(u64),
    External(UnstableArchiveId, UnstableTypeId),
}
