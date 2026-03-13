use hello_nestforge_graphql::{build_schema, AppConfig, AppModule};
use nestforge::prelude::*;

const PORT: u16 = 3001;

async fn bootstrap() -> anyhow::Result<()> {
    let factory = NestForgeFactory::<AppModule>::create()?;
    let config = factory.container().resolve::<AppConfig>()?;
    let schema = build_schema(config.app_name.clone());

    factory
        .with_graphql_config(schema, GraphQlConfig::new("/graphql").with_graphiql("/"))
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
