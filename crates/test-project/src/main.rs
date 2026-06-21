use skaf::StructureType;

// #[derive(Debug)]
#[derive(Debug, StructureType)]
#[skaf(name = "v1.deployment")]
struct Deployment {
    pub name: String,
}

fn main() {}
