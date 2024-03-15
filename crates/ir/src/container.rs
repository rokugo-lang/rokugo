use crate::op_code::IrInstruction;

pub struct IrContainer {
    _data: Vec<u8>,
}

impl IrContainer {
    /// # Safety
    /// This function can receive any data and it is up to the caller to ensure that the data is valid IR in valid
    /// version.
    pub unsafe fn from_vec(data: Vec<u8>) -> Self {
        IrContainer { _data: data }
    }

    pub fn iter(&self) -> IrContainerIterator {
        IrContainerIterator {
            _container: self,
            _index: 0,
        }
    }
}

impl<'container> IntoIterator for &'container IrContainer {
    type Item = IrInstruction<'container>;
    type IntoIter = IrContainerIterator<'container>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IrContainerIterator<'container> {
    _container: &'container IrContainer,
    _index: usize,
}

impl<'container> Iterator for IrContainerIterator<'container> {
    type Item = IrInstruction<'container>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
