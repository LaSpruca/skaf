use std::any::Any;

use crate::engine::Engine;

#[allow(dead_code)]
pub struct Proxy<T> {
    pub line: usize,
    pub col: usize,
    pub value: ProxyValue<T>,
}

impl<T> Proxy<T>
where
    T: Clone + 'static,
{
    pub fn get_value(&self, engine: &Engine) -> T {
        self.value.get_value(engine)
    }

    pub fn to_any(self) -> Proxy<Box<dyn Any>> {
        Proxy {
            line: self.line,
            col: self.col,
            value: match self.value {
                ProxyValue::Value(val) => ProxyValue::Value(Box::new(val)),
                ProxyValue::Query(items) => ProxyValue::Query(items),
                ProxyValue::Expression(name, items) => ProxyValue::Expression(name, items),
            },
        }
    }
}

impl Proxy<Box<dyn Any>> {
    pub fn get_value_as<T>(&self, engine: &Engine) -> T
    where
        T: Clone + 'static,
    {
        self.value.get_value_as(engine)
    }
}

pub enum ProxyValue<T> {
    Value(T),
    Query(Vec<String>),
    Expression(String, Vec<Proxy<Box<dyn Any>>>),
}

impl<T> ProxyValue<T>
where
    T: Clone + 'static,
{
    pub(self) fn get_value(&self, engine: &Engine) -> T {
        match self {
            Self::Value(t) => t.clone(),
            Self::Query(path) => engine
                .query(path)
                .downcast::<T>()
                .expect("Downcast failed")
                .as_ref()
                .clone(),
            Self::Expression(function, args) => {
                // println!("reslove {function}");
                engine
                    .call(function, args)
                    .downcast::<T>()
                    .expect("Downcast failed")
                    .as_ref()
                    .clone()
            }
        }
    }
}

impl ProxyValue<Box<dyn Any>> {
    pub fn get_value_as<T>(&self, engine: &Engine) -> T
    where
        T: Clone + 'static,
    {
        match self {
            Self::Value(t) => t.downcast_ref::<T>().expect("Downcast failed").clone(),
            Self::Query(path) => engine
                .query(path)
                .downcast::<T>()
                .expect("Downcast failed")
                .as_ref()
                .clone(),
            Self::Expression(function, args) => engine
                .call(function, args)
                .downcast::<T>()
                .expect("Downcast failed")
                .as_ref()
                .clone(),
        }
    }
}
