use std::any::TypeId;

use skaf::engine::{
    Engine, Function,
    proxy::Proxy,
    structure::{Structure, StructureType, StrutureProxy},
};

#[derive(Clone, Debug)]
struct Deployment {
    name: String,
    namespace: String,
}

struct DeploymentProxy {
    name: Proxy<String>,
    namespace: Proxy<String>,
}

impl StrutureProxy for DeploymentProxy {
    fn get(&self, path: &str, engine: &Engine) -> Option<Box<dyn std::any::Any>> {
        match path {
            "name" => Some(Box::new(self.name.get_value(engine))),
            "namespace" => Some(Box::new(self.namespace.get_value(engine))),
            _ => None,
        }
    }
}

impl StructureType for Deployment {
    type ProxyType = DeploymentProxy;

    fn make_proxy(object: skaf::parser::Object, engine: &Engine) -> Self::ProxyType {
        Self::ProxyType {
            name: engine.make_value_string(object.get_field("name").expect("Unreachable")),
            namespace: engine
                .make_value_string(object.get_field("namespace").expect("Unreachable")),
        }
    }

    fn get_structure() -> Structure {
        Structure::builder()
            .field::<String>("name")
            .field::<String>("namespace")
            .build()
    }

    fn make(proxy: &Self::ProxyType, engine: &Engine) -> Self {
        Self {
            name: proxy.name.get_value(engine),
            namespace: proxy.namespace.get_value(engine),
        }
    }

    fn tag() -> &'static str {
        return "v1.deployment";
    }
}

#[derive(Clone, Debug)]
struct Namespace {
    name: String,
}

struct NamespaceProxy {
    name: Proxy<String>,
}

impl StrutureProxy for NamespaceProxy {
    fn get(&self, path: &str, engine: &Engine) -> Option<Box<dyn std::any::Any>> {
        match path {
            "name" => Some(Box::new(self.name.get_value(engine))),
            _ => None,
        }
    }
}

impl StructureType for Namespace {
    type ProxyType = NamespaceProxy;

    fn make_proxy(object: skaf::parser::Object, engine: &Engine) -> Self::ProxyType {
        Self::ProxyType {
            name: engine.make_value_string(object.get_field("name").expect("Unreachable")),
        }
    }

    fn get_structure() -> Structure {
        Structure::builder().field::<String>("name").build()
    }

    fn make(proxy: &Self::ProxyType, engine: &Engine) -> Self {
        Self {
            name: proxy.name.get_value(engine),
        }
    }

    fn tag() -> &'static str {
        "namespace"
    }
}

const SRC: &str = include_str!("../test.skaf");

#[allow(non_camel_case_types)]
struct upperise {}
impl upperise {
    pub fn function(value: String) -> String {
        value.to_uppercase()
    }
}

impl Function for upperise {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "upperise"
    }

    fn sig(&self) -> (Vec<TypeId>, TypeId) {
        return (
            vec![std::any::TypeId::of::<String>()],
            std::any::TypeId::of::<String>(),
        );
    }

    fn call(
        &self,
        engine: &skaf::engine::Engine,
        args: &Vec<Proxy<Box<dyn std::any::Any>>>,
    ) -> Box<dyn std::any::Any> {
        let value: String = args[0].get_value_as(engine);

        Box::new(Self::function(value))
    }

    fn make_args(
        &self,
        values: &Vec<skaf::parser::Value>,
        engine: &Engine,
    ) -> Vec<Proxy<Box<dyn std::any::Any>>> {
        assert_eq!(values.len(), 1, "Should have 1 arg");
        return vec![engine.make_value_string(&values[0]).to_any()];
    }
}

fn main() {
    // let lex = Lexer::new(SRC);
    // for i in lex.clone() {
    //     println!("{i:?}");
    // }

    // let parsed = parse(lex);

    // for item in parsed.iter() {
    //     println!("{} {}", item.get_tag(), item.get_name());
    // }

    // println!("Parsed =========================\n{parsed:#?}");

    let mut engine = Engine::builder()
        .function(upperise {})
        .structure::<Namespace>()
        .structure::<Deployment>()
        .build();

    println!("Engine =========================");
    engine.load(SRC);
    println!("{engine:#?}");

    println!("othe-thing {:?}", engine.get::<Namespace>("other-thing"));
    println!(
        "edgelink-namespace {:?}",
        engine.get::<Namespace>("edgelink-namespace")
    );
    println!(
        "default-namespace {:?}",
        engine.get::<Namespace>("default-namespace")
    );
    println!(
        "edge-link-deployment {:?}",
        engine.get::<Deployment>("edge-link-deployment")
    );
}
