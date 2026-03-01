mod app_config;
mod app_module;
mod graphql;

use app_config::AppConfig;
use app_module::AppModule;
use graphql::schema::build_schema;
use nestforge::{GraphQlConfig, NestForgeFactory, NestForgeFactoryGraphQlExt};

const PORT: u16 = 3001;

async fn bootstrap() -> anyhow::Result<()> {
    let factory = NestForgeFactory::<AppModule>::create()?;
    let config = factory.container().resolve::<AppConfig>()?;
    let schema = build_schema(config.app_name.clone());

    factory
        .with_graphql_config(
            schema,
            GraphQlConfig::new("/graphql").with_graphiql("/"),
        )
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
