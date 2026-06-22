use skaf::{Engine, StructureType, function};

#[derive(Debug, StructureType)]
#[skaf(name = "v1.deployment")]
struct Deployment {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug, StructureType)]
#[skaf(name = "namespace")]
struct Namespace {
    pub name: String,
}

#[function]
fn upperise(value: String) -> String {
    value.to_uppercase()
}

fn main() {
    let mut engine = Engine::builder()
        .function(upperise)
        .structure::<Namespace>()
        .structure::<Deployment>()
        .build();

    engine.load(include_str!("../test.skaf"));

    println!(
        "default-namespace: {:?}",
        engine.get::<Namespace>("default-namespace")
    );
    println!(
        "edgelink-namespace: {:?}",
        engine.get::<Namespace>("edgelink-namespace")
    );
    println!(
        "edge-link-deployment: {:?}",
        engine.get::<Deployment>("edge-link-deployment")
    );
    println!("other-thing: {:?}", engine.get::<Namespace>("other-thing"));
}
