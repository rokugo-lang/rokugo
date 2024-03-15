use super::RegisterId;

pub mod memory;
pub mod naturals;

pub trait Register {
    fn id(&self) -> RegisterId;
}
