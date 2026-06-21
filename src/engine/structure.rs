use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{engine::Engine, parser::Object};

#[derive(Debug, Clone)]
pub struct StructureBuilder {
    inner: Structure,
}

impl StructureBuilder {
    pub fn new() -> Self {
        Self {
            inner: Structure {
                fields: HashMap::new(),
            },
        }
    }

    pub fn field<T: 'static + Clone>(mut self, name: impl Into<String>) -> Self {
        self.inner.fields.insert(name.into(), TypeId::of::<T>());
        self
    }

    pub fn build(self) -> Structure {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub struct Structure {
    fields: HashMap<String, TypeId>,
}

impl Structure {
    pub fn builder() -> StructureBuilder {
        StructureBuilder::new()
    }

    pub fn get_field(&self, name: impl AsRef<str>) -> Option<TypeId> {
        self.fields.get(name.as_ref()).map(|x| *x)
    }
}

pub trait StructureType
where
    Self::ProxyType: StrutureProxy + 'static,
{
    type ProxyType;

    fn make_proxy(object: Object, engine: &Engine) -> Self::ProxyType;
    fn get_structure() -> Structure;
    fn make(proxy: &Self::ProxyType, engine: &Engine) -> Self;
}

pub trait StrutureProxy: Any {
    fn get(&self, path: &str, engine: &Engine) -> Option<Box<dyn Any>>;
}
