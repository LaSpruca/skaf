#[derive(Proxied)]
struct Namespace {
    name: String,
}

struct Namespace__PROXY {
    name: Proxy<String>,
}

impl Proxied for Namespace {
    fn structure(&self) -> ObjectStruture {
        let mut structure = ObjectStruture::new();
        structure.field::<String>("name");
    }

    fn resolve(&self, obj: PreObj, ctx: EngineContext) {
        Ok(Self {
            name: ctx.resolve("name"),
        })
    }
}

#[derive(Proxy)]
struct Deployment {
    name: String,
    namespace: String,
}
