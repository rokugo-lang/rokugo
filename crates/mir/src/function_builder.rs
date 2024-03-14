use std::{mem, sync::OnceLock};

use rokugo_backend_common::{FunctionId, UnstableTypeId, ValueId};

use crate::emit::{content::MirContent, parameter::Parameter};

#[derive(Debug)]
pub struct FunctionBuilder {
    id: FunctionId,
    signature: FunctionSignature,
    mir: OnceLock<MirContent>,
}

impl FunctionBuilder {
    pub(crate) fn new<T: IntoIterator<Item = Parameter>>(
        id: FunctionId,
        signature_builder: FunctionSignatureBuilder<T>,
    ) -> Self {
        let mut parameters = Vec::new();
        for (index, parameter) in signature_builder.parameters.into_iter().enumerate() {
            let parameter_value_id = unsafe { mem::transmute(u32::MAX - index as u32) };
            parameters.push((parameter, parameter_value_id));
        }

        Self {
            id,
            signature: FunctionSignature {
                _parameters: parameters,
                _return_type: signature_builder.return_type,
            },
            mir: OnceLock::new(),
        }
    }

    pub fn id(&self) -> FunctionId {
        self.id
    }

    pub fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    pub fn mir(&self) -> Option<&MirContent> {
        self.mir.get()
    }

    pub fn set_or_update_mir(&self, mir: MirContent) {
        // TODO: Use another function to set the MIR, which will be allow to change the MIR once for every compilation.
        self.mir.set(mir).expect("MIR already set");
    }
}

#[derive(Debug)]
pub struct FunctionSignatureBuilder<T: IntoIterator<Item = Parameter> = [Parameter; 0]> {
    pub parameters: T,
    pub return_type: UnstableTypeId,
}

impl Default for FunctionSignatureBuilder {
    fn default() -> Self {
        Self {
            parameters: Default::default(),
            return_type: UnstableTypeId::VOID,
        }
    }
}

#[derive(Debug)]
pub struct FunctionSignature {
    _parameters: Vec<(Parameter, ValueId)>,
    _return_type: UnstableTypeId,
}
