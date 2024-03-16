use std::{
    ops::Deref,
    sync::{atomic::AtomicU64, Arc},
};

use dashmap::{mapref::one::Ref, DashMap};
use rokugo_backend_common::FunctionId;

use crate::{
    emit::parameter::Parameter,
    function_builder::{FunctionBuilder, FunctionSignatureBuilder},
};

#[derive(Clone, Debug)]
pub struct ArchiveBuilderRef {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    next_function_id: AtomicU64,
    functions: DashMap<FunctionId, FunctionBuilder>,
}

impl ArchiveBuilderRef {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                next_function_id: AtomicU64::new(0),
                functions: DashMap::new(),
            }),
        }
    }

    pub fn define_function<T: IntoIterator<Item = Parameter>>(
        &self,
        signature_builder: FunctionSignatureBuilder<T>,
    ) -> FunctionBuilderRef<'_> {
        let function_id = unsafe {
            std::mem::transmute(
                self.inner
                    .next_function_id
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            )
        };

        let function = FunctionBuilder::new(function_id, signature_builder);
        self.inner.functions.insert(function_id, function);
        FunctionBuilderRef {
            inner: self.inner.functions.get(&function_id).unwrap(),
        }
    }
}

impl Default for ArchiveBuilderRef {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FunctionBuilderRef<'a> {
    inner: Ref<'a, FunctionId, FunctionBuilder>,
}

impl Deref for FunctionBuilderRef<'_> {
    type Target = FunctionBuilder;

    fn deref(&self) -> &Self::Target {
        self.inner.value()
    }
}
