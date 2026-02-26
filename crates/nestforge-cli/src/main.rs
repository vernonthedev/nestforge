use anyhow::{bail, Context, Result};
use nestforge_db::{Db, DbConfig};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "new" => {
            let app_name = args
                .get(2)
                .context("Missing app name. Example: nestforge new demo-app")?;
            create_new_app(app_name)?;
        }
        "g" | "generate" => {
            let kind = args
                .get(2)
                .context("Missing generator kind. Example: nestforge g resource users")?;
            let name = args
                .get(3)
                .context("Missing generator name. Example: nestforge g resource users")?;

            match kind.as_str() {
                "resource" => generate_resource(name)?,
                "controller" => generate_controller_only(name)?,
                "service" => generate_service_only(name)?,
                "module" => generate_module(name)?,
                _ => bail!(
                    "Unknown generator `{}`. Use: resource | controller | service | module",
                    kind
                ),
            }
        }
        "db" => run_db_command(&args)?,
        "docs" => run_docs_command()?,
        "fmt" => run_fmt_command()?,
        _ => {
            print_help();
        }
    }

    Ok(())
}

fn print_help() {
    println!("NestForge CLI");
    println!();
    println!("Usage:");
    println!("  nestforge new <app-name>");
    println!("  nestforge g resource <name>");
    println!("  nestforge g controller <name>");
    println!("  nestforge g service <name>");
    println!("  nestforge g module <name>");
    println!("  nestforge db init");
    println!("  nestforge db generate <name>");
    println!("  nestforge db migrate");
    println!("  nestforge db status");
    println!("  nestforge docs");
    println!("  nestforge fmt");
    println!();
    println!("Install:");
    println!("  cargo install --path crates/nestforge-cli");
    println!();
    println!("Examples:");
    println!("  nestforge new care-api");
    println!("  nestforge g resource users");
    println!("  nestforge db init");
    println!("  nestforge db generate create_users_table");
}

/* ------------------------------
   NEW APP SCAFFOLD
------------------------------ */

fn create_new_app(app_name: &str) -> Result<()> {
    let app_dir = env::current_dir()?.join(app_name);

    if app_dir.exists() {
        bail!("App `{}` already exists at {}", app_name, app_dir.display());
    }

    /* Create folders */
    fs::create_dir_all(app_dir.join("src/controllers"))?;
    fs::create_dir_all(app_dir.join("src/services"))?;
    fs::create_dir_all(app_dir.join("src/dto"))?;

    /* Cargo.toml */
    write_file(
        &app_dir.join("Cargo.toml"),
        &template_app_cargo_toml(app_name),
    )?;

    /* main.rs */
    write_file(&app_dir.join("src/main.rs"), &template_main_rs())?;

    /* app_module.rs */
    write_file(
        &app_dir.join("src/app_module.rs"),
        &template_app_module_rs(),
    )?;

    /* controllers */
    write_file(
        &app_dir.join("src/controllers/mod.rs"),
        &template_controllers_mod_rs(),
    )?;
    write_file(
        &app_dir.join("src/controllers/app_controller.rs"),
        &template_app_controller_rs(),
    )?;
    write_file(
        &app_dir.join("src/controllers/health_controller.rs"),
        &template_health_controller_rs(),
    )?;

    /* services */
    write_file(
        &app_dir.join("src/services/mod.rs"),
        &template_services_mod_rs(),
    )?;
    write_file(
        &app_dir.join("src/services/app_config.rs"),
        &template_app_config_rs(),
    )?;

    /* dto */
    write_file(&app_dir.join("src/dto/mod.rs"), &template_dto_mod_rs())?;
    write_file(
        &app_dir.join(".env.example"),
        "DATABASE_URL=postgres://postgres:postgres@localhost/postgres\n",
    )?;
    write_file(
        &app_dir.join(".env"),
        "DATABASE_URL=postgres://postgres:postgres@localhost/postgres\n",
    )?;

    println!("Created NestForge app at {}", app_dir.display());
    println!();
    println!("Next:");
    println!("  cd {}", app_dir.display());
    println!("  cargo run");
    println!();
    println!("Then generate your first resource:");
    println!("  nestforge g resource users");

    Ok(())
}

/* ------------------------------
   DB COMMANDS
------------------------------ */

fn run_db_command(args: &[String]) -> Result<()> {
    let action = args
        .get(2)
        .context("Missing db command. Use: init | generate | migrate | status")?;
    let app_root = detect_app_root()?;

    match action.as_str() {
        "init" => db_init(&app_root),
        "generate" => {
            let name = args
                .get(3)
                .context("Missing migration name. Example: nestforge db generate create_users")?;
            db_generate(&app_root, name)
        }
        "migrate" => db_migrate(&app_root),
        "status" => db_status(&app_root),
        _ => bail!(
            "Unknown db command `{}`. Use: init | generate | migrate | status",
            action
        ),
    }
}

fn run_fmt_command() -> Result<()> {
    let target_dir = detect_app_root().or_else(|_| env::current_dir())?;
    let status = Command::new("cargo")
        .arg("fmt")
        .current_dir(&target_dir)
        .status()
        .with_context(|| format!("Failed to run cargo fmt in {}", target_dir.display()))?;

    if !status.success() {
        bail!("cargo fmt failed in {}", target_dir.display());
    }

    println!("Formatted Rust sources in {}", target_dir.display());
    Ok(())
}

fn run_docs_command() -> Result<()> {
    let app_root = detect_app_root().or_else(|_| env::current_dir())?;
    let docs_dir = app_root.join("docs");
    fs::create_dir_all(&docs_dir)?;
    let openapi_file = docs_dir.join("openapi.json");
    let skeleton = r#"{
  "openapi": "3.1.0",
  "info": {
    "title": "NestForge API",
    "version": "0.1.0"
  },
  "paths": {}
}
"#;
    write_file(&openapi_file, skeleton)?;
    println!("Generated OpenAPI skeleton at {}", openapi_file.display());
    Ok(())
}

fn db_init(app_root: &Path) -> Result<()> {
    fs::create_dir_all(migrations_dir(app_root))?;
    fs::create_dir_all(nestforge_dir(app_root))?;

    let applied = applied_migrations_file(app_root);
    if !applied.exists() {
        write_file(&applied, "")?;
    }

    let env_example = app_root.join(".env.example");
    if !env_example.exists() {
        write_file(
            &env_example,
            "DATABASE_URL=postgres://postgres:postgres@localhost/postgres\n",
        )?;
    }

    let env_file = app_root.join(".env");
    if !env_file.exists() {
        write_file(
            &env_file,
            "DATABASE_URL=postgres://postgres:postgres@localhost/postgres\n",
        )?;
    }

    println!("Initialized DB migration setup in {}", app_root.display());
    Ok(())
}

fn db_generate(app_root: &Path, name: &str) -> Result<()> {
    db_init(app_root)?;

    let slug = to_snake_case(name);
    let stamp = current_unix_timestamp()?;
    let file_name = format!("{stamp}_{slug}.sql");
    let file_path = migrations_dir(app_root).join(&file_name);

    if file_path.exists() {
        bail!("Migration already exists: {}", file_path.display());
    }

    let template = format!(
        "-- Migration: {name}\n-- Generated by nestforge db generate\n\n-- Write SQL statements below.\n-- Example:\n-- CREATE TABLE users (\n--   id BIGSERIAL PRIMARY KEY,\n--   email TEXT NOT NULL UNIQUE\n-- );\n"
    );
    write_file(&file_path, &template)?;

    println!("Created migration {}", file_name);
    Ok(())
}

fn db_migrate(app_root: &Path) -> Result<()> {
    db_init(app_root)?;

    let migrations = list_migration_files(app_root)?;
    let applied = read_applied_migrations(app_root)?;
    let applied_names: HashSet<String> = applied.keys().cloned().collect();
    let pending: Vec<PathBuf> = migrations
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|name| !applied_names.contains(name))
                .unwrap_or(false)
        })
        .collect();

    if pending.is_empty() {
        println!("No pending migrations.");
        return Ok(());
    }

    let database_url = resolve_database_url(app_root)?;
    let rt = tokio::runtime::Runtime::new().context("Failed to initialize tokio runtime")?;
    let db = rt
        .block_on(Db::connect(DbConfig::new(database_url.clone())))
        .with_context(|| format!("Failed to connect using DATABASE_URL `{database_url}`"))?;

    for migration in pending {
        let file_name = migration
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid migration filename")?
            .to_string();
        let sql = fs::read_to_string(&migration)
            .with_context(|| format!("Failed to read migration {}", migration.display()))?;
        let statements = split_sql_statements(&sql);

        if statements.is_empty() {
            println!("Skipping empty migration {}", file_name);
            let hash = compute_content_hash(&sql);
            append_applied_migration(app_root, &file_name, &hash)?;
            continue;
        }

        for stmt in statements {
            rt.block_on(db.execute(&stmt)).with_context(|| {
                format!("Migration {} failed on statement: {}", file_name, stmt)
            })?;
        }

        let hash = compute_content_hash(&sql);
        append_applied_migration(app_root, &file_name, &hash)?;
        println!("Applied {}", file_name);
    }

    println!("Migration run complete.");
    Ok(())
}

fn db_status(app_root: &Path) -> Result<()> {
    db_init(app_root)?;

    let migrations = list_migration_files(app_root)?;
    let applied = read_applied_migrations(app_root)?;

    if migrations.is_empty() {
        println!("No migration files found.");
        return Ok(());
    }

    let mut applied_count = 0usize;
    let mut pending_count = 0usize;

    let mut drift_count = 0usize;
    for migration in migrations {
        let file_name = migration
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid migration filename")?;
        if let Some(stored_hash) = applied.get(file_name) {
            let content = fs::read_to_string(&migration)
                .with_context(|| format!("Failed to read migration {}", migration.display()))?;
            let current_hash = compute_content_hash(&content);
            if stored_hash.is_empty() || *stored_hash == current_hash {
                applied_count += 1;
                println!("[applied] {file_name}");
            } else {
                drift_count += 1;
                println!("[drift]   {file_name} (applied hash differs from current file)");
            }
        } else {
            pending_count += 1;
            println!("[pending] {file_name}");
        }
    }

    println!();
    println!("Applied: {applied_count}");
    println!("Pending: {pending_count}");
    println!("Drift: {drift_count}");
    Ok(())
}

/* ------------------------------
   GENERATORS
------------------------------ */

fn generate_resource(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);

    generate_dto_files(&app_root, &resource, &singular, &pascal_singular)?;
    generate_service_file(
        &app_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
    )?;
    generate_controller_file(
        &app_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
    )?;

    patch_dto_mod(&app_root, &singular, &pascal_singular)?;
    patch_services_mod(&app_root, &resource, &pascal_plural)?;
    patch_controllers_mod(&app_root, &resource, &pascal_plural)?;
    patch_app_module(&app_root, &resource, &pascal_plural)?;

    println!("Generated resource `{}`", resource);
    Ok(())
}

fn generate_controller_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);

    generate_controller_file(
        &app_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
    )?;
    patch_controllers_mod(&app_root, &resource, &pascal_plural)?;
    patch_app_module_controllers_only(&app_root, &pascal_plural)?;

    println!("Generated controller `{}`", resource);
    Ok(())
}

fn generate_service_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);

    generate_service_file(
        &app_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
    )?;
    patch_services_mod(&app_root, &resource, &pascal_plural)?;
    patch_app_module_providers_only(&app_root, &pascal_plural)?;

    println!("Generated service `{}`", resource);
    Ok(())
}

fn generate_module(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let module_name = normalize_resource_name(name);
    let pascal_module = format!("{}Module", to_pascal_case(&module_name));
    let module_file = app_root
        .join("src")
        .join(format!("{}_module.rs", module_name));

    if module_file.exists() {
        bail!("Module already exists: {}", module_file.display());
    }

    write_file(
        &module_file,
        &template_feature_module_rs(&module_name, &pascal_module),
    )?;
    patch_main_mod_decl(&app_root, &module_name)?;
    patch_root_app_module_import(&app_root, &module_name, &pascal_module)?;
    patch_root_app_module_imports_list(&app_root, &pascal_module)?;

    println!("Generated module `{}`", module_name);
    Ok(())
}

/* ------------------------------
   FILE GENERATION
------------------------------ */

fn generate_dto_files(
    app_root: &Path,
    resource: &str,
    singular: &str,
    pascal_singular: &str,
) -> Result<()> {
    let dto_dir = app_root.join("src/dto");

    let entity_file = dto_dir.join(format!("{}_dto.rs", singular));
    let create_file = dto_dir.join(format!("create_{}_dto.rs", singular));
    let update_file = dto_dir.join(format!("update_{}_dto.rs", singular));

    if !entity_file.exists() {
        write_file(&entity_file, &template_entity_dto_rs(pascal_singular))?;
    }
    if !create_file.exists() {
        write_file(&create_file, &template_create_dto_rs(pascal_singular))?;
    }
    if !update_file.exists() {
        write_file(&update_file, &template_update_dto_rs(pascal_singular))?;
    }

    let _ = resource; // kept for future template customization
    Ok(())
}

fn generate_service_file(
    app_root: &Path,
    resource: &str,
    singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
) -> Result<()> {
    let service_path = app_root
        .join("src/services")
        .join(format!("{}_service.rs", resource));
    if service_path.exists() {
        println!("Service already exists: {}", service_path.display());
        return Ok(());
    }

    write_file(
        &service_path,
        &template_resource_service_rs(resource, singular, pascal_plural, pascal_singular),
    )?;
    Ok(())
}

fn generate_controller_file(
    app_root: &Path,
    resource: &str,
    singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
) -> Result<()> {
    let controller_path = app_root
        .join("src/controllers")
        .join(format!("{}_controller.rs", resource));

    if controller_path.exists() {
        println!("Controller already exists: {}", controller_path.display());
        return Ok(());
    }

    write_file(
        &controller_path,
        &template_resource_controller_rs(resource, singular, pascal_plural, pascal_singular),
    )?;
    Ok(())
}

/* ------------------------------
   PATCHERS
------------------------------ */

fn patch_dto_mod(app_root: &Path, singular: &str, pascal_singular: &str) -> Result<()> {
    let path = app_root.join("src/dto/mod.rs");
    let mut content = fs::read_to_string(&path)?;

    let mod_lines = [
        format!("pub mod {}_dto;", singular),
        format!("pub mod create_{}_dto;", singular),
        format!("pub mod update_{}_dto;", singular),
    ];

    let use_lines = [
        format!("pub use {}_dto::{}Dto;", singular, pascal_singular),
        format!(
            "pub use create_{}_dto::Create{}Dto;",
            singular, pascal_singular
        ),
        format!(
            "pub use update_{}_dto::Update{}Dto;",
            singular, pascal_singular
        ),
    ];

    for line in mod_lines {
        if !content.contains(&line) {
            content.push_str(&format!("\n{}", line));
        }
    }

    for line in use_lines {
        if !content.contains(&line) {
            content.push_str(&format!("\n{}", line));
        }
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_services_mod(app_root: &Path, resource: &str, pascal_plural: &str) -> Result<()> {
    let path = app_root.join("src/services/mod.rs");
    let mut content = fs::read_to_string(&path)?;

    let mod_line = format!("pub mod {}_service;", resource);
    let use_line = format!("pub use {}_service::{}Service;", resource, pascal_plural);

    if !content.contains(&mod_line) {
        content.push_str(&format!("\n{}", mod_line));
    }
    if !content.contains(&use_line) {
        content.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_controllers_mod(app_root: &Path, resource: &str, pascal_plural: &str) -> Result<()> {
    let path = app_root.join("src/controllers/mod.rs");
    let mut content = fs::read_to_string(&path)?;

    let mod_line = format!("pub mod {}_controller;", resource);
    let use_line = format!(
        "pub use {}_controller::{}Controller;",
        resource, pascal_plural
    );

    if !content.contains(&mod_line) {
        content.push_str(&format!("\n{}", mod_line));
    }
    if !content.contains(&use_line) {
        content.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_app_module(app_root: &Path, resource: &str, pascal_plural: &str) -> Result<()> {
    patch_app_module_controllers_only(app_root, pascal_plural)?;
    patch_app_module_providers_only(app_root, pascal_plural)?;

    /* Also patch imports */
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;

    /* controllers import */
    content = patch_brace_import_list(
        &content,
        "controllers",
        &format!("{}Controller", pascal_plural),
    );

    /* services import */
    content = patch_brace_import_list(&content, "services", &format!("{}Service", pascal_plural));

    fs::write(path, content)?;
    let _ = resource;
    Ok(())
}

fn patch_app_module_controllers_only(app_root: &Path, pascal_plural: &str) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;

    let marker = "/* nestforge:controllers */";
    let entry = format!("{}Controller,", pascal_plural);

    if content.contains(&entry) {
        return Ok(());
    }

    content = content.replace(marker, &format!("{}\n        {}", marker, entry));
    fs::write(path, content)?;
    Ok(())
}

fn patch_app_module_providers_only(app_root: &Path, pascal_plural: &str) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;

    let marker = "/* nestforge:providers */";
    let entry = format!("{}Service::new(),", pascal_plural);

    if content.contains(&entry) {
        return Ok(());
    }

    content = content.replace(marker, &format!("{}\n        {}", marker, entry));
    fs::write(path, content)?;
    Ok(())
}

/* ------------------------------
   TEMPLATE HELPERS
------------------------------ */

fn template_app_cargo_toml(app_name: &str) -> String {
    let framework_version = env!("CARGO_PKG_VERSION");

    format!(
        r#"[package]
name = "{app_name}"
version = "0.1.0"
edition = "2021"

[workspace]
members = []

[dependencies]
nestforge = "{framework_version}"
axum = "0.8"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
anyhow = "1"
"#
    )
}

fn template_main_rs() -> String {
    r#"mod app_module;
mod controllers;
mod dto;
mod services;

use app_module::AppModule;
use nestforge::NestForgeFactory;

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .with_global_prefix("api")
        .with_version("v1")
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
"#
    .to_string()
}

fn template_app_module_rs() -> String {
    r#"use nestforge::module;

use crate::{
    controllers::{AppController, HealthController},
    services::{AppConfig},
};

#[module(
    imports = [
        /* nestforge:imports */
    ],
    controllers = [
        AppController,
        HealthController,
        /* nestforge:controllers */
    ],
    providers = [
        AppConfig { app_name: "NestForge App".to_string() },
        /* nestforge:providers */
    ],
    exports = []
)]
pub struct AppModule;
"#
    .to_string()
}

fn template_controllers_mod_rs() -> String {
    r#"pub mod app_controller;
pub mod health_controller;

pub use app_controller::AppController;
pub use health_controller::HealthController;
"#
    .to_string()
}

fn template_services_mod_rs() -> String {
    r#"pub mod app_config;

pub use app_config::AppConfig;
"#
    .to_string()
}

fn template_dto_mod_rs() -> String {
    "/* DTO exports get generated here */\n".to_string()
}

fn template_app_controller_rs() -> String {
    r#"use nestforge::{controller, routes, HttpException, Inject};

use crate::services::AppConfig;

#[controller("")]
pub struct AppController;

#[routes]
impl AppController {
    #[nestforge::get("/")]
    async fn root(cfg: Inject<AppConfig>) -> Result<String, HttpException> {
        Ok(format!("Welcome to {}", cfg.app_name))
    }
}
"#
    .to_string()
}

fn template_health_controller_rs() -> String {
    r#"use nestforge::{controller, routes};

#[controller("")]
pub struct HealthController;

#[routes]
impl HealthController {
    #[nestforge::get("/health")]
    async fn health() -> String {
        "OK".to_string()
    }
}
"#
    .to_string()
}

fn template_app_config_rs() -> String {
    r#"#[derive(Clone)]
pub struct AppConfig {
    pub app_name: String,
}
"#
    .to_string()
}

fn template_entity_dto_rs(pascal_singular: &str) -> String {
    format!(
        r#"#[nestforge::dto]
pub struct {pascal_singular}Dto {{
    pub id: u64,
    pub name: String,
}}

nestforge::impl_identifiable!({pascal_singular}Dto, id);
"#
    )
}

fn template_create_dto_rs(pascal_singular: &str) -> String {
    format!(
        r#"#[nestforge::dto]
pub struct Create{pascal_singular}Dto {{
    #[validate(required)]
    pub name: String,
}}
"#
    )
}

fn template_update_dto_rs(pascal_singular: &str) -> String {
    format!(
        r#"#[nestforge::dto]
pub struct Update{pascal_singular}Dto {{
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}}
"#
    )
}

fn template_resource_service_rs(
    _resource: &str,
    _singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
) -> String {
    format!(
        r#"use nestforge::ResourceService;

use crate::dto::{pascal_singular}Dto;

pub type {pascal_plural}Service = ResourceService<{pascal_singular}Dto>;
"#
    )
}

fn template_resource_controller_rs(
    resource: &str,
    _singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
) -> String {
    format!(
        r#"use axum::Json;
use nestforge::{{controller, routes, ApiResult, Inject, List, OptionHttpExt, Param, ResultHttpExt, ValidatedBody}};

use crate::dto::{{Create{pascal_singular}Dto, Update{pascal_singular}Dto, {pascal_singular}Dto}};
use crate::services::{pascal_plural}Service;

#[controller("/{resource}")]
pub struct {pascal_plural}Controller;

#[routes]
impl {pascal_plural}Controller {{
    #[nestforge::get("/")]
    async fn list(
        service: Inject<{pascal_plural}Service>,
    ) -> ApiResult<List<{pascal_singular}Dto>> {{
        Ok(Json(service.all()))
    }}

    #[nestforge::get("/{{id}}")]
    async fn get_one(
        id: Param<u64>,
        service: Inject<{pascal_plural}Service>,
    ) -> ApiResult<{pascal_singular}Dto> {{
        let id = id.value();
        let item = service
            .get(id)
            .or_not_found_id("{pascal_singular}", id)?;

        Ok(Json(item))
    }}

    #[nestforge::post("/")]
    async fn create(
        service: Inject<{pascal_plural}Service>,
        body: ValidatedBody<Create{pascal_singular}Dto>,
    ) -> ApiResult<{pascal_singular}Dto> {{
        let item = service
            .create(body.value())
            .or_bad_request()?;
        Ok(Json(item))
    }}

    #[nestforge::put("/{{id}}")]
    async fn update(
        id: Param<u64>,
        service: Inject<{pascal_plural}Service>,
        body: ValidatedBody<Update{pascal_singular}Dto>,
    ) -> ApiResult<{pascal_singular}Dto> {{
        let id = id.value();
        let item = service
            .update(id, body.value())
            .or_bad_request()?
            .or_not_found_id("{pascal_singular}", id)?;

        Ok(Json(item))
    }}
}}
"#,
        resource = resource,
        pascal_plural = pascal_plural,
        pascal_singular = pascal_singular
    )
}

/* ------------------------------
   UTILS
------------------------------ */

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))
}

fn detect_app_root() -> Result<PathBuf> {
    let cwd = env::current_dir()?;

    /* If user is inside an app folder */
    if cwd.join("src").exists() && cwd.join("Cargo.toml").exists() {
        return Ok(cwd);
    }

    bail!("Run generator inside an app folder (where Cargo.toml + src/ exist).")
}

fn normalize_resource_name(name: &str) -> String {
    to_snake_case(name).replace(' ', "_")
}

fn singular_name(resource: &str) -> String {
    if resource.ends_with("ies") && resource.len() > 3 {
        format!("{}y", &resource[..resource.len() - 3])
    } else if resource.ends_with('s') && resource.len() > 1 {
        resource[..resource.len() - 1].to_string()
    } else {
        resource.to_string()
    }
}

fn to_pascal_case(input: &str) -> String {
    input
        .split(['_', '-', ' '])
        .filter(|s| !s.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::new();

    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            for c in ch.to_lowercase() {
                out.push(c);
            }
        } else if ch == '-' || ch == ' ' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }

    out
}

fn patch_brace_import_list(content: &str, module_name: &str, item: &str) -> String {
    let start_marker = format!("{module_name}::{{");

    if let Some(start_idx) = content.find(&start_marker) {
        if let Some(end_rel) = content[start_idx..].find("},") {
            let end_idx = start_idx + end_rel + 1; // points to the '}'
            let segment = &content[start_idx..=end_idx];

            if segment.contains(item) {
                return content.to_string();
            }

            let replaced = segment.replace("}", &format!(", {item}}}"));
            return content.replacen(segment, &replaced, 1);
        }
    }

    content.to_string()
}

fn template_feature_module_rs(module_name: &str, pascal_module: &str) -> String {
    format!(
        r#"use nestforge::module;

#[module(
    imports = [],
    controllers = [],
    providers = [],
    exports = []
)]
pub struct {pascal_module};

// Module: {module_name}
"#
    )
}

fn patch_root_app_module_import(
    app_root: &Path,
    module_name: &str,
    pascal_module: &str,
) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;
    let import_line = format!("use crate::{}_module::{};\n", module_name, pascal_module);

    if !content.contains(&import_line) {
        content = format!("{import_line}{content}");
        fs::write(path, content)?;
    }

    Ok(())
}

fn patch_root_app_module_imports_list(app_root: &Path, pascal_module: &str) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;
    let marker = "/* nestforge:imports */";
    let entry = format!("{pascal_module},");

    if content.contains(&entry) {
        return Ok(());
    }

    if content.contains(marker) {
        content = content.replace(marker, &format!("{marker}\n        {entry}"));
        fs::write(path, content)?;
        return Ok(());
    }

    if let Some(start) = content.find("imports = [") {
        let segment = &content[start..];
        if let Some(close_rel) = segment.find(']') {
            let close_idx = start + close_rel;
            content.insert_str(close_idx, &format!("\n        {entry}\n    "));
            fs::write(path, content)?;
        }
    }

    Ok(())
}

fn patch_main_mod_decl(app_root: &Path, module_name: &str) -> Result<()> {
    let path = app_root.join("src/main.rs");
    let mut content = fs::read_to_string(&path)?;
    let decl = format!("mod {}_module;", module_name);
    if content.contains(&decl) {
        return Ok(());
    }

    if let Some(idx) = content.find("mod app_module;") {
        let insert_at = idx + "mod app_module;".len();
        content.insert_str(insert_at, &format!("\n{decl}"));
        fs::write(path, content)?;
    }

    Ok(())
}

fn current_unix_timestamp() -> Result<u64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before Unix epoch")?;
    Ok(now.as_secs())
}

fn nestforge_dir(app_root: &Path) -> PathBuf {
    app_root.join(".nestforge")
}

fn migrations_dir(app_root: &Path) -> PathBuf {
    app_root.join("migrations")
}

fn applied_migrations_file(app_root: &Path) -> PathBuf {
    nestforge_dir(app_root).join("applied_migrations.txt")
}

fn list_migration_files(app_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let dir = migrations_dir(app_root);

    if !dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(&dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let is_sql = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("sql"))
            .unwrap_or(false);

        if is_sql {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn read_applied_migrations(app_root: &Path) -> Result<HashMap<String, String>> {
    let file = applied_migrations_file(app_root);
    if !file.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&file)?;
    let mut map = HashMap::new();
    for line in content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if let Some((name, hash)) = line.split_once('|') {
            map.insert(name.to_string(), hash.to_string());
        } else {
            map.insert(line.to_string(), String::new());
        }
    }
    Ok(map)
}

fn append_applied_migration(app_root: &Path, migration_file_name: &str, hash: &str) -> Result<()> {
    let file = applied_migrations_file(app_root);
    let mut open = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .with_context(|| format!("Failed to open {}", file.display()))?;
    writeln!(open, "{migration_file_name}|{hash}")?;
    Ok(())
}

fn resolve_database_url(app_root: &Path) -> Result<String> {
    if let Ok(url) = env::var("DATABASE_URL") {
        if !url.trim().is_empty() {
            return Ok(url);
        }
    }

    let env_path = app_root.join(".env");
    if env_path.exists() {
        let content = fs::read_to_string(&env_path)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(value) = trimmed.strip_prefix("DATABASE_URL=") {
                let clean = value.trim().trim_matches('"').trim_matches('\'');
                if !clean.is_empty() {
                    return Ok(clean.to_string());
                }
            }
        }
    }

    bail!(
        "DATABASE_URL not found. Set it in environment or add it to {}",
        env_path_or_default(app_root).display()
    )
}

fn env_path_or_default(app_root: &Path) -> PathBuf {
    app_root.join(".env")
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
        .map(|stmt| format!("{stmt};"))
        .collect()
}

fn compute_content_hash(content: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
