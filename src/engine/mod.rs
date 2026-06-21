use crate::{
    engine::{
        proxy::{Proxy, ProxyValue},
        structure::{Structure, StructureType, StrutureProxy},
    },
    parser::{Identifier, Object, Value, lex::Lexer, parse},
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

pub mod proxy;
pub mod structure;

pub struct EngineBuilder {
    functions: HashMap<String, Box<dyn Function>>,
    structures: HashMap<String, Structure>,
    create_proxies: HashMap<String, Box<dyn Fn(Object, &Engine) -> Box<dyn StrutureProxy>>>,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structures: HashMap::new(),
            create_proxies: HashMap::new(),
        }
    }

    pub fn function<T: Function + 'static>(mut self, function: T) -> Self {
        self.functions.insert(T::name().into(), Box::from(function));
        self
    }

    pub fn structure<T: StructureType>(mut self) -> Self {
        self.structures.insert(T::tag().into(), T::get_structure());
        self.create_proxies.insert(
            T::tag().into(),
            Box::new(|object, engine| Box::new(T::make_proxy(object, engine))),
        );
        self
    }

    pub fn build(self) -> Engine {
        Engine {
            functions: self.functions,
            structures: self.structures,
            create_proxies: self.create_proxies,
            objects: HashMap::new(),
        }
    }
}

pub struct Engine {
    functions: HashMap<String, Box<dyn Function>>,
    structures: HashMap<String, Structure>,
    create_proxies: HashMap<String, Box<dyn Fn(Object, &Engine) -> Box<dyn StrutureProxy>>>,
    objects: HashMap<Identifier, Box<dyn StrutureProxy>>,
}

impl Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field(
                "functions",
                &self
                    .functions
                    .iter()
                    .map(|x| x.0.clone())
                    .collect::<Vec<String>>(),
            )
            .field("structures", &self.structures)
            .finish()
    }
}

impl Engine {
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    pub fn get<T: StructureType>(&self, object: impl ToString) -> Option<T> {
        let proxy = self.objects.get(&Identifier {
            line: 0,
            col: 0,
            value: object.to_string(),
        })?;

        let k: &dyn Any = proxy.as_ref() as &dyn Any;

        Some(T::make(k.downcast_ref::<T::ProxyType>()?, self))
    }

    pub fn query(&self, path: &[String]) -> Box<dyn Any> {
        assert_eq!(path.len(), 2, "Query paths must currently be 2 sections");

        let Some(value) = self.objects.get(&Identifier {
            line: 0,
            col: 0,
            value: path[0].clone(),
        }) else {
            unreachable!()
        };

        let Some(value) = value.get(&path[1], self) else {
            unreachable!();
        };

        return value;
    }

    pub fn call(&self, function: &str, args: &Vec<Proxy<Box<dyn Any>>>) -> Box<dyn Any> {
        self.functions[function].call(self, args)
    }

    pub fn make_value_string(&self, value: &Value) -> Proxy<String> {
        match value.value {
            crate::parser::ValueVariant::String(ref v) => Proxy {
                line: value.line,
                col: value.col,
                value: ProxyValue::Value(v.clone()),
            },
            crate::parser::ValueVariant::Invoke(_, _) | crate::parser::ValueVariant::Query(_) => {
                self.make_value::<String>(value)
            }
        }
    }

    pub fn make_value<T>(&self, value: &Value) -> Proxy<T>
    where
        T: Clone + 'static,
    {
        Proxy {
            line: value.line,
            col: value.col,
            value: match value.value {
                crate::parser::ValueVariant::String(_) => {
                    unreachable!("Cannot construct non-primitave type");
                }
                crate::parser::ValueVariant::Invoke(ref function, ref values) => {
                    ProxyValue::Expression(
                        function.clone(),
                        self.functions[function].make_args(values, self),
                    )
                }
                crate::parser::ValueVariant::Query(ref items) => ProxyValue::Query(items.clone()),
            },
        }
    }

    pub fn load(&mut self, src: impl AsRef<str>) {
        let parsed = parse(Lexer::new(src.as_ref()));
        let mut tag_map = HashMap::with_capacity(parsed.len());

        // Generate tag map
        for object in parsed.iter() {
            let Some(s) = self.structures.get(object.get_tag()) else {
                println!(
                    "{}:{}: Struct {} has not been registered",
                    object.line,
                    object.col,
                    object.get_tag()
                );

                continue;
            };

            if tag_map.contains_key(object.get_name()) {
                println!(
                    "{}:{}: Duplicate objected named {}",
                    object.name.line,
                    object.name.col,
                    object.get_name()
                );

                continue;
            }

            tag_map.insert(
                object.get_name().to_string(),
                (object.get_tag().to_string(), s.clone()),
            );
        }

        'outer: for obj in parsed {
            let Some((name, s)) = tag_map.get(obj.get_name()) else {
                continue;
            };

            for (key, value) in obj.data.iter() {
                let Some(field) = s.get_field(&key.value) else {
                    println!("{}:{} field does not exist on struct", key.line, key.col);
                    continue 'outer;
                };

                if !self.check_type(field, value, &tag_map) {
                    continue 'outer;
                }
            }

            let Some(create_proxy) = self.create_proxies.get(name) else {
                unreachable!();
            };

            let name = obj.name.clone();

            let proxy = create_proxy(obj, self);

            self.objects.insert(name, proxy);
        }
    }

    fn check_type(
        &self,
        t: TypeId,
        value: &Value,
        tag_map: &HashMap<String, (String, Structure)>,
    ) -> bool {
        match &value.value {
            crate::parser::ValueVariant::String(_) => {
                if t != TypeId::of::<String>() {
                    println!("Wrong type, shouldn't be a string");
                    return false;
                }

                return true;
            }

            crate::parser::ValueVariant::Invoke(name, args) => {
                let Some(fun) = self.functions.get(name) else {
                    println!(
                        "{}:{}: Function {} does not exist",
                        value.line, value.col, name
                    );

                    return false;
                };

                let (sig_args, ret) = fun.sig();

                if ret != t {
                    println!("{}:{} {} returns wrong type", value.line, value.col, name);
                    return false;
                }

                if sig_args.len() != args.len() {
                    println!(
                        "{}:{} {} takes {} args, {} provided",
                        value.line,
                        value.col,
                        name,
                        sig_args.len(),
                        args.len()
                    );

                    return false;
                }

                for (i, (sig_arg, arg)) in sig_args.iter().zip(args.iter()).enumerate() {
                    if !self.check_type(*sig_arg, arg, tag_map) {
                        println!("{}:{}: {i}th arg to {name} is wrong", value.line, value.col,);
                        return false;
                    }
                }

                return true;
            }
            crate::parser::ValueVariant::Query(items) => {
                if items.len() != 2 {
                    println!(
                        "{}:{} Currently query strings must contain exactly two segments",
                        value.line, value.col
                    );
                    return false;
                }

                let Some((name, q)) = tag_map.get(items[0].as_str()) else {
                    println!("{}:{} {} does not exist", value.line, value.col, items[0]);
                    return false;
                };

                let Some(target_field) = q.get_field(&items[1]) else {
                    println!(
                        "{}:{} Struct {} does have a field {}",
                        value.line, value.col, name, items[1]
                    );

                    return false;
                };

                if target_field != t {
                    println!(
                        "{}:{} Field {} is the wrong type",
                        value.line, value.col, items[1]
                    );

                    return false;
                }

                return true;
            }
        }
    }
}

pub trait Function {
    fn name() -> &'static str
    where
        Self: Sized;

    fn sig(&self) -> (Vec<TypeId>, TypeId);
    fn call(&self, engine: &Engine, args: &Vec<Proxy<Box<dyn Any>>>) -> Box<dyn Any>;
    fn make_args(&self, values: &Vec<Value>, engine: &Engine) -> Vec<Proxy<Box<dyn Any>>>;
}
