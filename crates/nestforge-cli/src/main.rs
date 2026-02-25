use anyhow::{bail, Context, Result};
use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "new" => {
            let app_name = args.get(2).context("Missing app name. Example: nestforge new demo-app")?;
            create_new_app(app_name)?;
        }
        "g" | "generate" => {
            let kind = args.get(2).context("Missing generator kind. Example: nestforge g resource users")?;
            let name = args.get(3).context("Missing generator name. Example: nestforge g resource users")?;

            match kind.as_str() {
                "resource" => generate_resource(name)?,
                "controller" => generate_controller_only(name)?,
                "service" => generate_service_only(name)?,
                _ => bail!("Unknown generator `{}`. Use: resource | controller | service", kind),
            }
        }
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
    println!();
    println!("Examples:");
    println!("  nestforge new care-api");
    println!("  nestforge g resource users");
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
    write_file(&app_dir.join("src/app_module.rs"), &template_app_module_rs())?;

    /* controllers */
    write_file(&app_dir.join("src/controllers/mod.rs"), &template_controllers_mod_rs())?;
    write_file(
        &app_dir.join("src/controllers/app_controller.rs"),
        &template_app_controller_rs(),
    )?;
    write_file(
        &app_dir.join("src/controllers/health_controller.rs"),
        &template_health_controller_rs(),
    )?;

    /* services */
    write_file(&app_dir.join("src/services/mod.rs"), &template_services_mod_rs())?;
    write_file(
        &app_dir.join("src/services/app_config.rs"),
        &template_app_config_rs(),
    )?;

    /* dto */
    write_file(&app_dir.join("src/dto/mod.rs"), &template_dto_mod_rs())?;

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
   GENERATORS
------------------------------ */

fn generate_resource(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);

    generate_dto_files(&app_root, &resource, &singular, &pascal_singular)?;
    generate_service_file(&app_root, &resource, &singular, &pascal_plural, &pascal_singular)?;
    generate_controller_file(&app_root, &resource, &singular, &pascal_plural, &pascal_singular)?;

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

    generate_controller_file(&app_root, &resource, &singular, &pascal_plural, &pascal_singular)?;
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

    generate_service_file(&app_root, &resource, &singular, &pascal_plural, &pascal_singular)?;
    patch_services_mod(&app_root, &resource, &pascal_plural)?;
    patch_app_module_providers_only(&app_root, &pascal_plural)?;

    println!("Generated service `{}`", resource);
    Ok(())
}

/* ------------------------------
   FILE GENERATION
------------------------------ */

fn generate_dto_files(app_root: &Path, resource: &str, singular: &str, pascal_singular: &str) -> Result<()> {
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
    let service_path = app_root.join("src/services").join(format!("{}_service.rs", resource));
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
        format!("pub use create_{}_dto::Create{}Dto;", singular, pascal_singular),
        format!("pub use update_{}_dto::Update{}Dto;", singular, pascal_singular),
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
    let use_line = format!("pub use {}_controller::{}Controller;", resource, pascal_plural);

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
    content = patch_brace_import_list(
        &content,
        "services",
        &format!("{}Service", pascal_plural),
    );

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .listen(3000)
        .await
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
    controllers = [
        AppController,
        HealthController,
        /* nestforge:controllers */
    ],
    providers = [
        AppConfig { app_name: "NestForge App".to_string() },
        /* nestforge:providers */
    ]
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
    async fn health() -> &'static str {
        "OK"
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
        r#"use nestforge::Identifiable;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct {pascal_singular}Dto {{
    pub id: u64,
    pub name: String,
}}

impl Identifiable for {pascal_singular}Dto {{
    fn id(&self) -> u64 {{
        self.id
    }}

    fn set_id(&mut self, id: u64) {{
        self.id = id;
    }}
}}
"#
    )
}

fn template_create_dto_rs(pascal_singular: &str) -> String {
    format!(
        r#"use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Create{pascal_singular}Dto {{
    pub name: String,
}}

impl Create{pascal_singular}Dto {{
    pub fn validate(&self) -> Result<(), String> {{
        if self.name.trim().is_empty() {{
            return Err("name is required".to_string());
        }}

        Ok(())
    }}
}}
"#
    )
}

fn template_update_dto_rs(pascal_singular: &str) -> String {
    format!(
        r#"use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Update{pascal_singular}Dto {{
    pub name: Option<String>,
}}

impl Update{pascal_singular}Dto {{
    pub fn validate(&self) -> Result<(), String> {{
        if self.name.is_none() {{
            return Err("at least one field is required".to_string());
        }}

        if let Some(name) = &self.name {{
            if name.trim().is_empty() {{
                return Err("name cannot be empty".to_string());
            }}
        }}

        Ok(())
    }}
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
        r#"use nestforge::InMemoryStore;

use crate::dto::{{Create{pascal_singular}Dto, Update{pascal_singular}Dto, {pascal_singular}Dto}};

#[derive(Clone)]
pub struct {pascal_plural}Service {{
    store: InMemoryStore<{pascal_singular}Dto>,
}}

impl {pascal_plural}Service {{
    pub fn new() -> Self {{
        Self {{
            store: InMemoryStore::new(),
        }}
    }}

    pub fn find_all(&self) -> Vec<{pascal_singular}Dto> {{
        self.store.find_all()
    }}

    pub fn find_by_id(&self, id: u64) -> Option<{pascal_singular}Dto> {{
        self.store.find_by_id(id)
    }}

    pub fn create(&self, dto: Create{pascal_singular}Dto) -> {pascal_singular}Dto {{
        let item = {pascal_singular}Dto {{
            id: 0,
            name: dto.name,
        }};

        self.store.create(item)
    }}

    pub fn update(&self, id: u64, dto: Update{pascal_singular}Dto) -> Option<{pascal_singular}Dto> {{
        self.store.update_by_id(id, |item| {{
            if let Some(name) = dto.name.clone() {{
                item.name = name;
            }}
        }})
    }}
}}
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
use nestforge::{{controller, routes, Body, HttpException, Inject, Param}};

use crate::dto::{{Create{pascal_singular}Dto, Update{pascal_singular}Dto, {pascal_singular}Dto}};
use crate::services::{pascal_plural}Service;

#[controller("/{resource}")]
pub struct {pascal_plural}Controller;

#[routes]
impl {pascal_plural}Controller {{
    #[nestforge::get("/")]
    async fn list(
        service: Inject<{pascal_plural}Service>,
    ) -> Result<Json<Vec<{pascal_singular}Dto>>, HttpException> {{
        Ok(Json(service.find_all()))
    }}

    #[nestforge::get("/{{id}}")]
    async fn get_one(
        id: Param<u64>,
        service: Inject<{pascal_plural}Service>,
    ) -> Result<Json<{pascal_singular}Dto>, HttpException> {{
        let item = service
            .find_by_id(*id)
            .ok_or_else(|| HttpException::not_found(format!("{pascal_singular} with id {{}} not found", *id)))?;

        Ok(Json(item))
    }}

    #[nestforge::post("/")]
    async fn create(
        service: Inject<{pascal_plural}Service>,
        body: Body<Create{pascal_singular}Dto>,
    ) -> Result<Json<{pascal_singular}Dto>, HttpException> {{
        body.validate().map_err(HttpException::bad_request)?;
        Ok(Json(service.create(body.into_inner())))
    }}

    #[nestforge::put("/{{id}}")]
    async fn update(
        id: Param<u64>,
        service: Inject<{pascal_plural}Service>,
        body: Body<Update{pascal_singular}Dto>,
    ) -> Result<Json<{pascal_singular}Dto>, HttpException> {{
        body.validate().map_err(HttpException::bad_request)?;

        let item = service
            .update(*id, body.into_inner())
            .ok_or_else(|| HttpException::not_found(format!("{pascal_singular} with id {{}} not found", *id)))?;

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
