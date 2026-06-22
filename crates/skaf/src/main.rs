use skaf::{
    StructureType,
    engine::{Engine, Function, proxy::Proxy},
};
use std::any::TypeId;

#[derive(Clone, Debug, StructureType)]
#[skaf(name = "v1.deployment")]
struct Deployment {
    name: String,
    namespace: String,
}

#[derive(Clone, Debug, StructureType)]
#[skaf(name = "namespace")]
struct Namespace {
    name: String,
}

const SRC: &str = include_str!("../../../test.skaf");

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
        (
            vec![std::any::TypeId::of::<String>()],
            std::any::TypeId::of::<String>(),
        )
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
        vec![engine.make_value_string(&values[0]).to_any()]
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
