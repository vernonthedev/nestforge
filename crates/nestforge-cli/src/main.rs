use anyhow::{bail, Context, Result};
use nestforge_db::{Db, DbConfig};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppTransport {
    Http,
    Graphql,
    Grpc,
    Microservices,
    Websockets,
}

impl AppTransport {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "http" => Ok(Self::Http),
            "graphql" => Ok(Self::Graphql),
            "grpc" => Ok(Self::Grpc),
            "microservices" | "ms" => Ok(Self::Microservices),
            "websockets" | "ws" => Ok(Self::Websockets),
            _ => bail!("Unknown transport `{value}`. Use: http | graphql | grpc | microservices | websockets"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Http => "HTTP",
            Self::Graphql => "GraphQL",
            Self::Grpc => "gRPC",
            Self::Microservices => "Microservices",
            Self::Websockets => "WebSocket",
        }
    }
}

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
            let transport = parse_new_transport_arg(&args[3..])?;
            create_new_app(app_name, transport)?;
        }
        "g" | "generate" => {
            let kind = args
                .get(2)
                .context("Missing generator kind. Example: nestforge g resource users")?;
            let name = args
                .get(3)
                .context("Missing generator name. Example: nestforge g resource users")?;
            let target_module = parse_target_module_arg(&args[4..])?;

            match kind.as_str() {
                "resource" => generate_resource(name, target_module.as_deref())?,
                "controller" => generate_controller_only(name, target_module.as_deref())?,
                "service" => generate_service_only(name, target_module.as_deref())?,
                "module" => generate_module(name)?,
                "guard" => generate_guard_only(name)?,
                "filter" => generate_exception_filter_only(name)?,
                "middleware" => generate_middleware_only(name)?,
                "interceptor" => generate_interceptor_only(name)?,
                "serializer" => generate_serializer_only(name)?,
                "graphql" => generate_graphql_resolver_only(name)?,
                "grpc" => generate_grpc_service_only(name)?,
                "gateway" => generate_websocket_gateway_only(name)?,
                "microservice" => generate_microservice_patterns_only(name)?,
                _ => bail!(
                    "Unknown generator `{}`. Use: resource | controller | service | module | guard | filter | middleware | interceptor | serializer | graphql | grpc | gateway | microservice",
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
    println!("  nestforge new <app-name> --transport <http|graphql|grpc|microservices|websockets>");
    println!("  nestforge g resource <name>");
    println!("  nestforge g controller <name>");
    println!("  nestforge g service <name>");
    println!("  nestforge g resource <name> --module <feature>");
    println!("  nestforge g module <name>");
    println!("  nestforge g guard <name>");
    println!("  nestforge g filter <name>");
    println!("  nestforge g middleware <name>");
    println!("  nestforge g interceptor <name>");
    println!("  nestforge g serializer <name>");
    println!("  nestforge g graphql <name>");
    println!("  nestforge g grpc <name>");
    println!("  nestforge g gateway <name>");
    println!("  nestforge g microservice <name>");
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
    println!("  nestforge new catalog-api --transport graphql");
    println!("  nestforge new greeter-service --transport grpc");
    println!("  nestforge new app-bus --transport microservices");
    println!("  nestforge new realtime-events --transport websockets");
    println!("  nestforge g resource users");
    println!("  nestforge g filter rewrite_bad_request");
    println!("  nestforge g middleware audit");
    println!("  nestforge g serializer user");
    println!("  nestforge g graphql users");
    println!("  nestforge g grpc billing");
    println!("  nestforge g gateway events");
    println!("  nestforge g microservice users");
    println!("  nestforge db init");
    println!("  nestforge db generate create_users_table");
}

/* ------------------------------
   NEW APP SCAFFOLD
------------------------------ */

fn create_new_app(app_name: &str, transport: AppTransport) -> Result<()> {
    let app_dir = env::current_dir()?.join(app_name);

    if app_dir.exists() {
        bail!("App `{}` already exists at {}", app_name, app_dir.display());
    }

    fs::create_dir_all(app_dir.join("src/services"))?;

    /* Cargo.toml */
    write_file(
        &app_dir.join("Cargo.toml"),
        &template_app_cargo_toml(app_name, resolve_nestforge_dependency_line(transport), transport),
    )?;

    /* main.rs */
    write_file(&app_dir.join("src/main.rs"), &template_main_rs(transport))?;

    /* app_module.rs */
    write_file(
        &app_dir.join("src/app_module.rs"),
        &template_app_module_rs(transport),
    )?;

    /* services */
    write_file(
        &app_dir.join("src/services/mod.rs"),
        &template_services_mod_rs(),
    )?;
    write_file(
        &app_dir.join("src/services/app_config.rs"),
        &template_app_config_rs(transport),
    )?;

    scaffold_transport_files(&app_dir, transport)?;

    write_file(
        &app_dir.join(".env.example"),
        &template_env_file(app_name, transport),
    )?;
    write_file(
        &app_dir.join(".env"),
        &template_env_file(app_name, transport),
    )?;

    println!(
        "Created NestForge {} app at {}",
        transport.label(),
        app_dir.display()
    );
    println!();
    println!("Next:");
    println!("  cd {}", app_dir.display());
    println!("  cargo run");

    if matches!(transport, AppTransport::Http) {
        println!();
        println!("Then generate your first resource:");
        println!("  nestforge g resource users");
    }

    Ok(())
}

fn scaffold_transport_files(app_dir: &Path, transport: AppTransport) -> Result<()> {
    match transport {
        AppTransport::Http => {
            fs::create_dir_all(app_dir.join("src/controllers"))?;
            fs::create_dir_all(app_dir.join("src/dto"))?;
            fs::create_dir_all(app_dir.join("src/guards"))?;
            fs::create_dir_all(app_dir.join("src/interceptors"))?;

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
            write_file(
                &app_dir.join("src/guards/mod.rs"),
                &template_guards_mod_rs(),
            )?;
            write_file(
                &app_dir.join("src/filters/mod.rs"),
                &template_filters_mod_rs(),
            )?;
            write_file(
                &app_dir.join("src/interceptors/mod.rs"),
                &template_interceptors_mod_rs(),
            )?;
            write_file(&app_dir.join("src/dto/mod.rs"), &template_dto_mod_rs())?;
        }
        AppTransport::Graphql => {
            fs::create_dir_all(app_dir.join("src/graphql"))?;
            write_file(&app_dir.join("src/graphql/mod.rs"), "pub mod schema;\n")?;
            write_file(
                &app_dir.join("src/graphql/schema.rs"),
                &template_graphql_schema_rs(),
            )?;
        }
        AppTransport::Grpc => {
            fs::create_dir_all(app_dir.join("src/grpc"))?;
            fs::create_dir_all(app_dir.join("proto"))?;
            write_file(&app_dir.join("build.rs"), &template_grpc_build_rs())?;
            write_file(
                &app_dir.join("proto/greeter.proto"),
                &template_grpc_proto(),
            )?;
            write_file(&app_dir.join("src/grpc/mod.rs"), &template_grpc_mod_rs())?;
            write_file(
                &app_dir.join("src/grpc/service.rs"),
                &template_grpc_service_rs(),
            )?;
        }
        AppTransport::Microservices => {
            fs::create_dir_all(app_dir.join("src/microservices"))?;
            write_file(
                &app_dir.join("src/microservices/mod.rs"),
                &template_microservices_app_mod_rs(),
            )?;
            write_file(
                &app_dir.join("src/microservices/app_patterns.rs"),
                &template_microservices_app_patterns_rs(),
            )?;
        }
        AppTransport::Websockets => {
            fs::create_dir_all(app_dir.join("src/ws"))?;
            write_file(&app_dir.join("src/ws/mod.rs"), &template_ws_mod_rs())?;
            write_file(
                &app_dir.join("src/ws/events_gateway.rs"),
                &template_ws_gateway_rs(),
            )?;
        }
    }

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
            "# Set your local database connection string before running DB commands.\nDATABASE_URL=postgres://<user>:<password>@localhost/<database>\n",
        )?;
    }

    let env_file = app_root.join(".env");
    if !env_file.exists() {
        write_file(
            &env_file,
            "# Set your local database connection string before running the app.\nDATABASE_URL=postgres://<user>:<password>@localhost/<database>\n",
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
        .block_on(Db::connect(DbConfig::new(database_url)))
        .context("Failed to connect using DATABASE_URL (value redacted)")?;

    for migration in pending {
        let file_name = migration
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid migration filename")?
            .to_string();
        let sql = fs::read_to_string(&migration)
            .with_context(|| format!("Failed to read migration {}", migration.display()))?;

        if !contains_sql_content(&sql) {
            println!("Skipping empty migration {}", file_name);
            let hash = compute_content_hash(&sql);
            append_applied_migration(app_root, &file_name, &hash)?;
            continue;
        }

        rt.block_on(async {
            let mut tx = db
                .begin()
                .await
                .with_context(|| format!("Migration {} failed to start transaction", file_name))?;

            tx.execute_script(&sql)
                .await
                .with_context(|| format!("Migration {} failed while executing SQL script", file_name))?;

            tx.commit()
                .await
                .with_context(|| format!("Migration {} failed to commit transaction", file_name))
        })?;

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

fn generate_resource(name: &str, target_module: Option<&str>) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);

    let target_root = generator_target_root(&app_root, target_module)?;
    let namespace = generator_namespace(target_module);

    generate_dto_files(&target_root, &resource, &singular, &pascal_singular)?;
    generate_service_file(
        &target_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &namespace,
    )?;
    generate_controller_file(
        &target_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &namespace,
    )?;

    patch_dto_mod(&target_root, &singular, &pascal_singular)?;
    patch_services_mod(&target_root, &resource, &pascal_plural)?;
    patch_controllers_mod(&target_root, &resource, &pascal_plural)?;

    if let Some(module_name) = target_module {
        patch_feature_module(&app_root, module_name, &pascal_plural)?;
    } else {
        patch_app_module(&app_root, &resource, &pascal_plural)?;
    }

    println!("Generated resource `{}`", resource);
    Ok(())
}

fn generate_controller_only(name: &str, target_module: Option<&str>) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);
    let target_root = generator_target_root(&app_root, target_module)?;
    let namespace = generator_namespace(target_module);

    generate_controller_file(
        &target_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &namespace,
    )?;
    patch_controllers_mod(&target_root, &resource, &pascal_plural)?;
    if let Some(module_name) = target_module {
        patch_feature_module(&app_root, module_name, &pascal_plural)?;
    } else {
        patch_app_module_controllers_only(&app_root, &pascal_plural)?;
    }

    println!("Generated controller `{}`", resource);
    Ok(())
}

fn generate_service_only(name: &str, target_module: Option<&str>) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);
    let target_root = generator_target_root(&app_root, target_module)?;
    let namespace = generator_namespace(target_module);

    generate_service_file(
        &target_root,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &namespace,
    )?;
    patch_services_mod(&target_root, &resource, &pascal_plural)?;
    if let Some(module_name) = target_module {
        patch_feature_module(&app_root, module_name, &pascal_plural)?;
    } else {
        patch_app_module_providers_only(&app_root, &pascal_plural)?;
    }

    println!("Generated service `{}`", resource);
    Ok(())
}

fn generate_module(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let module_name = normalize_resource_name(name);
    let pascal_module = format!("{}Module", to_pascal_case(&module_name));
    let module_dir = app_root.join("src").join(&module_name);
    let module_file = module_dir.join("mod.rs");

    if module_file.exists() {
        bail!("Module folder already exists: {}", module_dir.display());
    }

    write_file(
        &module_dir.join("mod.rs"),
        &template_feature_mod_rs(&module_name, &pascal_module),
    )?;
    write_file(
        &module_dir.join("controllers/mod.rs"),
        &template_feature_controllers_mod_rs(&module_name, &pascal_module),
    )?;
    write_file(
        &module_dir.join("controllers/controller.rs"),
        &template_feature_controller_rs(&module_name, &pascal_module),
    )?;
    write_file(
        &module_dir.join("services/mod.rs"),
        &template_feature_services_mod_rs(&module_name, &pascal_module),
    )?;
    write_file(
        &module_dir.join("services/service.rs"),
        &template_feature_service_rs(&module_name, &pascal_module),
    )?;
    write_file(
        &module_dir.join("dto/mod.rs"),
        &template_feature_dto_mod_rs(),
    )?;

    patch_main_mod_decl(&app_root, &module_name)?;
    patch_root_app_module_import(&app_root, &module_name, &pascal_module)?;
    patch_root_app_module_imports_list(&app_root, &pascal_module)?;

    println!("Generated module `{}`", module_name);
    Ok(())
}

fn generate_guard_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let guard_name = normalize_resource_name(name);
    let pascal_guard = format!("{}Guard", to_pascal_case(&guard_name));
    let guard_file = app_root
        .join("src/guards")
        .join(format!("{}_guard.rs", guard_name));

    if guard_file.exists() {
        println!("Guard already exists: {}", guard_file.display());
        return Ok(());
    }

    fs::create_dir_all(app_root.join("src/guards"))?;
    write_file(&guard_file, &template_guard_rs(&pascal_guard))?;
    patch_guards_mod(&app_root, &guard_name, &pascal_guard)?;
    patch_main_mod_decl(&app_root, "guards")?;

    println!("Generated guard `{}`", guard_name);
    Ok(())
}

fn generate_exception_filter_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let filter_name = normalize_resource_name(name);
    let pascal_filter = format!("{}Filter", to_pascal_case(&filter_name));
    let filter_file = app_root
        .join("src/filters")
        .join(format!("{}_filter.rs", filter_name));

    if filter_file.exists() {
        println!("Exception filter already exists: {}", filter_file.display());
        return Ok(());
    }

    fs::create_dir_all(app_root.join("src/filters"))?;
    write_file(&filter_file, &template_exception_filter_rs(&pascal_filter))?;
    patch_filters_mod(&app_root, &filter_name, &pascal_filter)?;
    patch_main_mod_decl(&app_root, "filters")?;

    println!("Generated exception filter `{}`", filter_name);
    Ok(())
}

fn generate_middleware_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let middleware_name = normalize_resource_name(name);
    let pascal_middleware = format!("{}Middleware", to_pascal_case(&middleware_name));
    let middleware_file = app_root
        .join("src/middleware")
        .join(format!("{}_middleware.rs", middleware_name));

    if middleware_file.exists() {
        println!("Middleware already exists: {}", middleware_file.display());
        return Ok(());
    }

    fs::create_dir_all(app_root.join("src/middleware"))?;
    write_file(
        &middleware_file,
        &template_middleware_rs(&pascal_middleware),
    )?;
    patch_middleware_mod(&app_root, &middleware_name, &pascal_middleware)?;
    patch_main_mod_decl(&app_root, "middleware")?;

    println!("Generated middleware `{}`", middleware_name);
    Ok(())
}

fn generate_interceptor_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let interceptor_name = normalize_resource_name(name);
    let pascal_interceptor = format!("{}Interceptor", to_pascal_case(&interceptor_name));
    let interceptor_file = app_root
        .join("src/interceptors")
        .join(format!("{}_interceptor.rs", interceptor_name));

    if interceptor_file.exists() {
        println!("Interceptor already exists: {}", interceptor_file.display());
        return Ok(());
    }

    fs::create_dir_all(app_root.join("src/interceptors"))?;
    write_file(
        &interceptor_file,
        &template_interceptor_rs(&pascal_interceptor),
    )?;
    patch_interceptors_mod(&app_root, &interceptor_name, &pascal_interceptor)?;
    patch_main_mod_decl(&app_root, "interceptors")?;

    println!("Generated interceptor `{}`", interceptor_name);
    Ok(())
}

fn generate_serializer_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let serializer_name = normalize_resource_name(name);
    let pascal_serializer = format!("{}Serializer", to_pascal_case(&serializer_name));
    let serializer_file = app_root
        .join("src/serializers")
        .join(format!("{}_serializer.rs", serializer_name));

    fs::create_dir_all(app_root.join("src/serializers"))?;

    if !serializer_file.exists() {
        write_file(
            &serializer_file,
            &template_serializer_rs(&serializer_name, &pascal_serializer),
        )?;
    } else {
        println!("Serializer already exists: {}", serializer_file.display());
    }

    patch_serializers_mod(&app_root, &serializer_name, &pascal_serializer)?;
    patch_main_mod_decl(&app_root, "serializers")?;

    println!("Generated serializer `{}`", serializer_name);
    println!("Next: implement `ResponseSerializer<T>` for your domain type");
    Ok(())
}

fn generate_websocket_gateway_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let gateway_name = normalize_resource_name(name);
    let pascal_gateway = format!("{}Gateway", to_pascal_case(&gateway_name));
    let gateway_file = app_root
        .join("src/ws")
        .join(format!("{}_gateway.rs", gateway_name));

    fs::create_dir_all(app_root.join("src/ws"))?;
    ensure_ws_mod(&app_root)?;

    if !gateway_file.exists() {
        write_file(
            &gateway_file,
            &template_named_ws_gateway_rs(&pascal_gateway),
        )?;
    } else {
        println!("WebSocket gateway already exists: {}", gateway_file.display());
    }

    patch_ws_mod(&app_root, &gateway_name, &pascal_gateway)?;
    patch_main_mod_decl(&app_root, "ws")?;

    println!("Generated WebSocket gateway `{}`", gateway_name);
    Ok(())
}

fn generate_graphql_resolver_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let resolver_name = normalize_resource_name(name);
    let pascal_name = to_pascal_case(&resolver_name);
    let graphql_dir = app_root.join("src/graphql");
    let resolver_file = graphql_dir.join(format!("{}_resolver.rs", resolver_name));

    fs::create_dir_all(&graphql_dir)?;
    ensure_graphql_mod(&app_root)?;

    if !resolver_file.exists() {
        write_file(
            &resolver_file,
            &template_graphql_resolver_rs(&resolver_name, &pascal_name),
        )?;
    } else {
        println!("GraphQL resolver already exists: {}", resolver_file.display());
    }

    patch_graphql_mod(&app_root, &resolver_name, &pascal_name)?;

    println!("Generated GraphQL resolver `{}`", resolver_name);
    println!("Next: wire `{pascal_name}Resolver` into src/graphql/schema.rs");
    Ok(())
}

fn generate_grpc_service_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let service_name = normalize_resource_name(name);
    let pascal_name = to_pascal_case(&service_name);
    let proto_path = app_root.join("proto").join(format!("{service_name}.proto"));
    let service_path = app_root
        .join("src/grpc")
        .join(format!("{}_service.rs", service_name));

    fs::create_dir_all(app_root.join("proto"))?;
    fs::create_dir_all(app_root.join("src/grpc"))?;
    ensure_grpc_build_rs(&app_root)?;
    ensure_grpc_mod(&app_root)?;

    if !proto_path.exists() {
        write_file(&proto_path, &template_named_grpc_proto(&service_name, &pascal_name))?;
    }

    if !service_path.exists() {
        write_file(
            &service_path,
            &template_named_grpc_service_rs(&service_name, &pascal_name),
        )?;
    } else {
        println!("gRPC service already exists: {}", service_path.display());
    }

    patch_grpc_build_rs(&app_root, &service_name)?;
    patch_grpc_mod_rs(&app_root, &service_name, &pascal_name)?;

    println!("Generated gRPC service `{}`", service_name);
    println!(
        "Next: mount `{pascal_name}ServiceServer::new({pascal_name}GrpcService::new(ctx))` in src/main.rs"
    );
    Ok(())
}

fn generate_microservice_patterns_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let pattern_name = normalize_resource_name(name);
    let pascal_name = to_pascal_case(&pattern_name);
    let patterns_dir = app_root.join("src/microservices");
    let patterns_file = patterns_dir.join(format!("{}_patterns.rs", pattern_name));

    fs::create_dir_all(&patterns_dir)?;
    ensure_microservices_mod(&app_root)?;

    if !patterns_file.exists() {
        write_file(
            &patterns_file,
            &template_microservice_patterns_rs(&pattern_name, &pascal_name),
        )?;
    } else {
        println!(
            "Microservice patterns already exist: {}",
            patterns_file.display()
        );
    }

    patch_microservices_mod(&app_root, &pattern_name, &pascal_name)?;
    patch_main_mod_decl(&app_root, "microservices")?;

    println!("Generated microservice patterns `{}`", pattern_name);
    println!(
        "Next: register `{pascal_name}Patterns::registry()` with your transport adapter or module provider"
    );
    println!("Note: enable the `microservices` feature on `nestforge` in Cargo.toml if needed");
    Ok(())
}

/* ------------------------------
   FILE GENERATION
------------------------------ */

fn generate_dto_files(
    target_root: &Path,
    resource: &str,
    singular: &str,
    pascal_singular: &str,
) -> Result<()> {
    let dto_dir = target_root.join("dto");

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
    target_root: &Path,
    resource: &str,
    singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    namespace: &str,
) -> Result<()> {
    let service_path = target_root
        .join("services")
        .join(format!("{}_service.rs", resource));
    if service_path.exists() {
        println!("Service already exists: {}", service_path.display());
        return Ok(());
    }

    write_file(
        &service_path,
        &template_resource_service_rs(
            resource,
            singular,
            pascal_plural,
            pascal_singular,
            namespace,
        ),
    )?;
    Ok(())
}

fn generate_controller_file(
    target_root: &Path,
    resource: &str,
    singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    namespace: &str,
) -> Result<()> {
    let controller_path = target_root
        .join("controllers")
        .join(format!("{}_controller.rs", resource));

    if controller_path.exists() {
        println!("Controller already exists: {}", controller_path.display());
        return Ok(());
    }

    write_file(
        &controller_path,
        &template_resource_controller_rs(
            resource,
            singular,
            pascal_plural,
            pascal_singular,
            namespace,
        ),
    )?;
    Ok(())
}

/* ------------------------------
   PATCHERS
------------------------------ */

fn patch_dto_mod(target_root: &Path, singular: &str, pascal_singular: &str) -> Result<()> {
    let path = target_root.join("dto/mod.rs");
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

fn patch_services_mod(target_root: &Path, resource: &str, pascal_plural: &str) -> Result<()> {
    let path = target_root.join("services/mod.rs");
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

fn patch_controllers_mod(target_root: &Path, resource: &str, pascal_plural: &str) -> Result<()> {
    let path = target_root.join("controllers/mod.rs");
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

fn patch_guards_mod(app_root: &Path, guard_name: &str, pascal_guard: &str) -> Result<()> {
    let path = app_root.join("src/guards/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_guards_mod_rs()
    };

    let mod_line = format!("pub mod {}_guard;", guard_name);
    let use_line = format!("pub use {}_guard::{};", guard_name, pascal_guard);

    if !content.contains(&mod_line) {
        content.push_str(&format!("\n{}", mod_line));
    }
    if !content.contains(&use_line) {
        content.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_interceptors_mod(
    app_root: &Path,
    interceptor_name: &str,
    pascal_interceptor: &str,
) -> Result<()> {
    let path = app_root.join("src/interceptors/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_interceptors_mod_rs()
    };

    let mod_line = format!("pub mod {}_interceptor;", interceptor_name);
    let use_line = format!(
        "pub use {}_interceptor::{};",
        interceptor_name, pascal_interceptor
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

fn patch_filters_mod(app_root: &Path, filter_name: &str, pascal_filter: &str) -> Result<()> {
    let path = app_root.join("src/filters/mod.rs");
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_filters_mod_rs()
    };
    let mod_line = format!("pub mod {}_filter;", filter_name);
    let use_line = format!("pub use {}_filter::{};", filter_name, pascal_filter);
    let mut next = content;

    if !next.contains(&mod_line) {
        next.push_str(&format!("\n{}", mod_line));
    }
    if !next.contains(&use_line) {
        next.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, next)?;
    Ok(())
}

fn patch_middleware_mod(
    app_root: &Path,
    middleware_name: &str,
    pascal_middleware: &str,
) -> Result<()> {
    let path = app_root.join("src/middleware/mod.rs");
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_middleware_mod_rs()
    };
    let mod_line = format!("pub mod {}_middleware;", middleware_name);
    let use_line = format!(
        "pub use {}_middleware::{};",
        middleware_name, pascal_middleware
    );
    let mut next = content;

    if !next.contains(&mod_line) {
        next.push_str(&format!("\n{}", mod_line));
    }
    if !next.contains(&use_line) {
        next.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, next)?;
    Ok(())
}

fn patch_serializers_mod(
    app_root: &Path,
    serializer_name: &str,
    pascal_serializer: &str,
) -> Result<()> {
    let path = app_root.join("src/serializers/mod.rs");
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_serializers_mod_rs()
    };
    let mod_line = format!("pub mod {}_serializer;", serializer_name);
    let use_line = format!(
        "pub use {}_serializer::{};",
        serializer_name, pascal_serializer
    );
    let mut next = content;

    if !next.contains(&mod_line) {
        next.push_str(&format!("\n{}", mod_line));
    }
    if !next.contains(&use_line) {
        next.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, next)?;
    Ok(())
}

fn ensure_graphql_mod(app_root: &Path) -> Result<()> {
    let path = app_root.join("src/graphql/mod.rs");
    if !path.exists() {
        write_file(&path, "pub mod schema;\n")?;
    }
    Ok(())
}

fn ensure_ws_mod(app_root: &Path) -> Result<()> {
    let path = app_root.join("src/ws/mod.rs");
    if !path.exists() {
        write_file(&path, &template_ws_mod_rs())?;
    }
    Ok(())
}

fn ensure_microservices_mod(app_root: &Path) -> Result<()> {
    let path = app_root.join("src/microservices/mod.rs");
    if !path.exists() {
        write_file(&path, &template_microservices_mod_rs())?;
    }
    Ok(())
}

fn patch_graphql_mod(app_root: &Path, resolver_name: &str, pascal_name: &str) -> Result<()> {
    let path = app_root.join("src/graphql/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        "pub mod schema;\n".to_string()
    };

    let mod_line = format!("pub mod {}_resolver;", resolver_name);
    let use_line = format!("pub use {}_resolver::{}Resolver;", resolver_name, pascal_name);

    if !content.contains(&mod_line) {
        content.push_str(&format!("\n{}", mod_line));
    }
    if !content.contains(&use_line) {
        content.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_ws_mod(app_root: &Path, gateway_name: &str, pascal_name: &str) -> Result<()> {
    let path = app_root.join("src/ws/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_ws_mod_rs()
    };

    let mod_line = format!("mod {}_gateway;", gateway_name);
    let use_line = format!("pub use {}_gateway::{};", gateway_name, pascal_name);

    if !content.contains(&mod_line) {
        content.push_str(&format!("\n{}", mod_line));
    }
    if !content.contains(&use_line) {
        content.push_str(&format!("\n{}", use_line));
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_microservices_mod(app_root: &Path, pattern_name: &str, pascal_name: &str) -> Result<()> {
    let path = app_root.join("src/microservices/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_microservices_mod_rs()
    };

    let mod_line = format!("pub mod {}_patterns;", pattern_name);
    let use_line = format!(
        "pub use {}_patterns::{}Patterns;",
        pattern_name, pascal_name
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

fn ensure_grpc_build_rs(app_root: &Path) -> Result<()> {
    let path = app_root.join("build.rs");
    if !path.exists() {
        write_file(&path, &template_grpc_build_rs())?;
    }
    Ok(())
}

fn ensure_grpc_mod(app_root: &Path) -> Result<()> {
    let path = app_root.join("src/grpc/mod.rs");
    if !path.exists() {
        write_file(&path, &template_grpc_mod_rs())?;
    }
    Ok(())
}

fn patch_grpc_build_rs(app_root: &Path, service_name: &str) -> Result<()> {
    let path = app_root.join("build.rs");
    let mut content = fs::read_to_string(&path)?;
    let proto_entry = format!("\"proto/{service_name}.proto\"");
    let rerun_line = format!("    println!(\"cargo:rerun-if-changed=proto/{service_name}.proto\");\n");

    if !content.contains(&proto_entry) {
        let target = ".compile_protos(&[\"proto/greeter.proto\"], &[\"proto\"])?;";
        if content.contains(target) {
            content = content.replace(
                target,
                &format!(
                    ".compile_protos(&[\"proto/greeter.proto\", {proto_entry}], &[\"proto\"])?;"
                ),
            );
        } else if let Some(start) = content.find(".compile_protos(&[") {
            if let Some(end_rel) = content[start..].find("], &[\"proto\"])?;") {
                let insert_at = start + end_rel;
                content.insert_str(insert_at, &format!(", {proto_entry}"));
            }
        }
    }

    if !content.contains(&format!("cargo:rerun-if-changed=proto/{service_name}.proto")) {
        if let Some(insert_at) = content.rfind("    Ok(())") {
            content.insert_str(insert_at, &rerun_line);
        }
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_grpc_mod_rs(app_root: &Path, service_name: &str, pascal_name: &str) -> Result<()> {
    let path = app_root.join("src/grpc/mod.rs");
    let mut content = fs::read_to_string(&path)?;
    let proto_module = format!(
        "    pub mod {service_name} {{\n        nestforge::tonic::include_proto!(\"{service_name}\");\n    }}\n"
    );
    let service_mod_line = format!("pub mod {}_service;", service_name);
    let service_use_line = format!(
        "pub use {}_service::{}GrpcService;",
        service_name, pascal_name
    );

    if content.contains("nestforge::tonic::include_proto!(\"hello\");") {
        content = content.replace(
            "pub mod proto {\n    nestforge::tonic::include_proto!(\"hello\");\n}\n\npub mod service;\n",
            "pub mod proto {\n    pub mod hello {\n        nestforge::tonic::include_proto!(\"hello\");\n    }\n}\n\npub mod service;\n",
        );
    }

    if !content.contains(&format!("pub mod {service_name} {{")) {
        if let Some(insert_at) = content.find("}\n\npub mod service;") {
            content.insert_str(insert_at, &proto_module);
        } else if let Some(insert_at) = content.find("}\n") {
            content.insert_str(insert_at, &proto_module);
        }
    }

    if !content.contains(&service_mod_line) {
        content.push_str(&format!("\n{}", service_mod_line));
    }
    if !content.contains(&service_use_line) {
        content.push_str(&format!("\n{}", service_use_line));
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

fn patch_feature_module(app_root: &Path, module_name: &str, pascal_plural: &str) -> Result<()> {
    let path = app_root.join("src").join(module_name).join("mod.rs");
    let mut content = fs::read_to_string(&path)?;

    let controller_use_marker = "/* nestforge:feature_controllers_use */";
    let provider_use_marker = "/* nestforge:feature_services_use */";
    let controllers_marker = "/* nestforge:feature_controllers */";
    let providers_marker = "/* nestforge:feature_providers */";
    let exports_marker = "/* nestforge:feature_exports */";

    let controller_use = format!("{}Controller,", pascal_plural);
    let service_use = format!("{}Service,", pascal_plural);
    let controller_entry = format!("{}Controller,", pascal_plural);
    let provider_entry = format!("{}Service::new(),", pascal_plural);
    let export_entry = format!("{}Service,", pascal_plural);

    let controller_use_block = format!("{}\n    {}", controller_use_marker, controller_use);
    let service_use_block = format!("{}\n    {}", provider_use_marker, service_use);
    let controller_block = format!("{}\n        {}", controllers_marker, controller_entry);
    let provider_block = format!("{}\n        {}", providers_marker, provider_entry);
    let export_block = format!("{}\n        {}", exports_marker, export_entry);

    if !content.contains(&controller_use_block) && content.contains(controller_use_marker) {
        content = content.replace(
            controller_use_marker,
            &format!("{}\n    {}", controller_use_marker, controller_use),
        );
    }
    if !content.contains(&service_use_block) && content.contains(provider_use_marker) {
        content = content.replace(
            provider_use_marker,
            &format!("{}\n    {}", provider_use_marker, service_use),
        );
    }
    if !content.contains(&controller_block) && content.contains(controllers_marker) {
        content = content.replace(
            controllers_marker,
            &format!("{}\n        {}", controllers_marker, controller_entry),
        );
    }
    if !content.contains(&provider_block) && content.contains(providers_marker) {
        content = content.replace(
            providers_marker,
            &format!("{}\n        {}", providers_marker, provider_entry),
        );
    }
    if !content.contains(&export_block) && content.contains(exports_marker) {
        content = content.replace(
            exports_marker,
            &format!("{}\n        {}", exports_marker, export_entry),
        );
    }

    fs::write(path, content)?;
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

fn parse_new_transport_arg(args: &[String]) -> Result<AppTransport> {
    if args.is_empty() {
        return Ok(AppTransport::Http);
    }

    if args.len() == 2 && args[0] == "--transport" {
        return AppTransport::parse(&args[1]);
    }

    bail!("Invalid new app options. Use: nestforge new <app-name> --transport <http|graphql|grpc|microservices|websockets>")
}

fn resolve_nestforge_dependency_line(transport: AppTransport) -> String {
    let framework_version = env!("CARGO_PKG_VERSION");
    let features = match transport {
        AppTransport::Http => "\"config\"",
        AppTransport::Graphql => "\"config\", \"graphql\"",
        AppTransport::Grpc => "\"config\", \"grpc\"",
        AppTransport::Microservices => "\"config\", \"microservices\", \"testing\"",
        AppTransport::Websockets => "\"config\", \"websockets\"",
    };
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("nestforge"));

    if let Some(path) = local_path {
        if path.exists() {
            let normalized = path.to_string_lossy().replace('\\', "/");
            return format!(
                "nestforge = {{ path = \"{}\", features = [{}] }}",
                normalized, features
            );
        }
    }

    format!(
        "nestforge = {{ version = \"{}\", features = [{}] }}",
        framework_version, features
    )
}

fn template_app_cargo_toml(
    app_name: &str,
    nestforge_dep: String,
    transport: AppTransport,
) -> String {
    let package_extra = if matches!(transport, AppTransport::Grpc) {
        "build = \"build.rs\"\n"
    } else {
        ""
    };

    let dependency_lines = match transport {
        AppTransport::Http => {
            "axum = \"0.8\"\ntokio = { version = \"1\", features = [\"full\"] }\nserde = { version = \"1\", features = [\"derive\"] }\nanyhow = \"1\"\n"
        }
        AppTransport::Graphql => {
            "tokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\n"
        }
        AppTransport::Grpc => {
            "tokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\ntonic = { version = \"0.12\", features = [\"transport\"] }\nprost = \"0.13\"\n"
        }
        AppTransport::Microservices => {
            "tokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\nserde = { version = \"1\", features = [\"derive\"] }\nserde_json = \"1\"\n"
        }
        AppTransport::Websockets => {
            "tokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\n"
        }
    };

    let build_dependencies = if matches!(transport, AppTransport::Grpc) {
        "\n[build-dependencies]\ntonic-build = \"0.12\"\n"
    } else {
        ""
    };

    format!(
        r#"[package]
name = "{app_name}"
version = "0.1.0"
edition = "2021"
{package_extra}

[dependencies]
{nestforge_dep}
{dependency_lines}{build_dependencies}
"#,
        nestforge_dep = nestforge_dep,
        package_extra = package_extra,
        dependency_lines = dependency_lines,
        build_dependencies = build_dependencies,
    )
}

fn template_main_rs(transport: AppTransport) -> String {
    match transport {
        AppTransport::Http => r#"mod app_module;
mod controllers;
mod dto;
mod guards;
mod interceptors;
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
        .to_string(),
        AppTransport::Graphql => r#"mod app_module;
mod graphql;
mod services;

use app_module::AppModule;
use graphql::schema::build_schema;
use nestforge::{GraphQlConfig, NestForgeFactory, NestForgeFactoryGraphQlExt};

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {
    let factory = NestForgeFactory::<AppModule>::create()?;
    let config = factory.container().resolve::<crate::services::AppConfig>()?;
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
"#
        .to_string(),
        AppTransport::Grpc => r#"mod app_module;
mod grpc;
mod services;

use app_module::AppModule;
use grpc::{proto::hello::greeter_server::GreeterServer, service::GreeterGrpcService};
use nestforge::NestForgeGrpcFactory;

const ADDR: &str = "127.0.0.1:50051";

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeGrpcFactory::<AppModule>::create()?
        .with_addr(ADDR)
        .listen_with(|ctx, addr| async move {
            nestforge::tonic::transport::Server::builder()
                .add_service(GreeterServer::new(GreeterGrpcService::new(ctx)))
                .serve(addr)
                .await
        })
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
"#
          .to_string(),
        AppTransport::Microservices => r#"mod app_module;
mod microservices;
mod services;

use app_module::AppModule;
use microservices::AppPatterns;
use nestforge::{MicroserviceClient, TestFactory, TransportMetadata};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let module = TestFactory::<AppModule>::create().build()?;
    let patterns = module.resolve::<AppPatterns>()?;
    let client = module.microservice_client_with_metadata(
        patterns.registry().clone(),
        "app-cli",
        TransportMetadata::new().insert("source", "scaffold"),
    );

    let response: serde_json::Value = client
        .send(
            "app.ping",
            serde_json::json!({{
                "name": "NestForge"
            }}),
        )
        .await?;

    println!("{}", serde_json::to_string_pretty(&response)?);
    module.shutdown()?;
    Ok(())
}
"#
        .to_string(),
        AppTransport::Websockets => r#"mod app_module;
mod services;
mod ws;

use app_module::AppModule;
use nestforge::{NestForgeFactory, NestForgeFactoryWebSocketExt};
use ws::EventsGateway;

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {
    NestForgeFactory::<AppModule>::create()?
        .with_websocket_gateway(EventsGateway)
        .listen(PORT)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}
"#
        .to_string(),
      }
  }

fn template_app_module_rs(transport: AppTransport) -> String {
    match transport {
        AppTransport::Http => r#"use nestforge::{module, ConfigModule, ConfigOptions};

use crate::{
    controllers::{AppController, HealthController},
    services::{AppConfig},
};

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(ConfigOptions::new().env_file(".env"))?)
}

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
        load_app_config()?,
        /* nestforge:providers */
    ],
    exports = []
)]
pub struct AppModule;
"#
        .to_string(),
        AppTransport::Graphql | AppTransport::Grpc | AppTransport::Websockets => r#"use nestforge::{module, ConfigModule, ConfigOptions};

use crate::services::AppConfig;

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(ConfigOptions::new().env_file(".env"))?)
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?],
    exports = [AppConfig]
)]
pub struct AppModule;
"#
        .to_string(),
        AppTransport::Microservices => r#"use nestforge::{module, ConfigModule, ConfigOptions};

use crate::{
    microservices::AppPatterns,
    services::AppConfig,
};

fn load_app_config() -> anyhow::Result<AppConfig> {
    Ok(ConfigModule::for_root::<AppConfig>(ConfigOptions::new().env_file(".env"))?)
}

fn load_patterns() -> anyhow::Result<AppPatterns> {
    Ok(AppPatterns::new())
}

#[module(
    imports = [],
    controllers = [],
    providers = [load_app_config()?, load_patterns()?],
    exports = [AppConfig, AppPatterns]
)]
pub struct AppModule;
"#
        .to_string(),
    }
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

fn template_guards_mod_rs() -> String {
    "/* Guard exports get generated here */\n".to_string()
}

fn template_middleware_mod_rs() -> String {
    "/* Middleware exports get generated here */\n".to_string()
}

fn template_filters_mod_rs() -> String {
    "/* Exception filter exports get generated here */\n".to_string()
}

fn template_interceptors_mod_rs() -> String {
    "/* Interceptor exports get generated here */\n".to_string()
}

fn template_serializers_mod_rs() -> String {
    "/* Serializer exports get generated here */\n".to_string()
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

fn template_app_config_rs(transport: AppTransport) -> String {
    let default_app_name = match transport {
        AppTransport::Http => "NestForge HTTP",
        AppTransport::Graphql => "NestForge GraphQL",
        AppTransport::Grpc => "NestForge gRPC",
        AppTransport::Websockets => "NestForge WebSockets",
    };

    format!(
        r#"#[derive(Clone)]
pub struct AppConfig {
    pub app_name: String,
}

impl nestforge::FromEnv for AppConfig {
    fn from_env(env: &nestforge::EnvStore) -> Result<Self, nestforge::ConfigError> {
        let app_name = env.get("APP_NAME").unwrap_or("{default_app_name}").to_string();
        Ok(Self { app_name })
    }
}
"#
    )
}

fn template_graphql_schema_rs() -> String {
    r#"use nestforge::async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema(app_name: String) -> AppSchema {
    Schema::build(QueryRoot { app_name }, EmptyMutation, EmptySubscription).finish()
}

pub struct QueryRoot {
    app_name: String,
}

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "ok"
    }

    async fn app_name(&self) -> &str {
        &self.app_name
    }
}
"#
    .to_string()
}

fn template_graphql_resolver_rs(resolver_name: &str, pascal_name: &str) -> String {
    let field_name = format!("{}_status", resolver_name);
    format!(
        r#"use nestforge::async_graphql::Object;

pub struct {pascal_name}Resolver;

#[Object]
impl {pascal_name}Resolver {{
    async fn {field_name}(&self) -> &str {{
        "ok"
    }}
}}
"#
    )
}

fn template_grpc_build_rs() -> String {
    r#"fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/greeter.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/greeter.proto");
    Ok(())
}
"#
    .to_string()
}

fn template_grpc_proto() -> String {
    r#"syntax = "proto3";

package hello;

service Greeter {
  rpc SayHello (HelloRequest) returns (HelloReply);
}

message HelloRequest {
  string name = 1;
}

message HelloReply {
  string message = 1;
}
"#
    .to_string()
}

fn template_named_grpc_proto(service_name: &str, pascal_name: &str) -> String {
    format!(
        r#"syntax = "proto3";

package {service_name};

service {pascal_name}Service {{
  rpc Get{pascal_name}Status ({pascal_name}StatusRequest) returns ({pascal_name}StatusReply);
}}

message {pascal_name}StatusRequest {{
  string name = 1;
}}

message {pascal_name}StatusReply {{
  string message = 1;
}}
"#
    )
}

fn template_grpc_mod_rs() -> String {
    r#"pub mod proto {
    pub mod hello {
        nestforge::tonic::include_proto!("hello");
    }
}

pub mod service;
"#
    .to_string()
}

fn template_grpc_service_rs() -> String {
    r#"use nestforge::{
    tonic::{Request, Response, Status},
    GrpcContext,
};

use crate::{
    grpc::proto::hello::{greeter_server::Greeter, HelloReply, HelloRequest},
    services::AppConfig,
};

#[derive(Clone)]
pub struct GreeterGrpcService {
    ctx: GrpcContext,
}

fn template_ws_mod_rs() -> String {
    r#"mod events_gateway;

pub use events_gateway::EventsGateway;
"#
    .to_string()
}

fn template_microservices_app_mod_rs() -> String {
    r#"mod app_patterns;

pub use app_patterns::AppPatterns;
"#
    .to_string()
}

fn template_microservices_app_patterns_rs() -> String {
    r#"#[derive(Clone)]
pub struct AppPatterns {
    registry: nestforge::MicroserviceRegistry,
}

impl AppPatterns {
    pub fn new() -> Self {
        Self {
            registry: nestforge::MicroserviceRegistry::builder()
                .message("app.ping", |payload: serde_json::Value, ctx| async move {
                    let config = ctx.resolve::<crate::services::AppConfig>()?;
                    Ok(serde_json::json!({
                        "app_name": config.app_name,
                        "received": payload,
                        "transport": ctx.transport(),
                    }))
                })
                .build(),
        }
    }

    pub fn registry(&self) -> &nestforge::MicroserviceRegistry {
        &self.registry
    }
}
"#
    .to_string()
}

fn template_microservices_mod_rs() -> String {
    "/* Microservice pattern exports get generated here */\n".to_string()
}

fn template_ws_gateway_rs() -> String {
    r#"use crate::services::AppConfig;
use nestforge::{Message, WebSocket, WebSocketContext, WebSocketGateway};

pub struct EventsGateway;

impl WebSocketGateway for EventsGateway {
    fn on_connect(
        &self,
        ctx: WebSocketContext,
        mut socket: WebSocket,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            let app_name = ctx
                .resolve::<AppConfig>()
                .map(|config| config.app_name.clone())
                .unwrap_or_else(|_| "NestForge WebSockets".to_string());

            let _ = socket
                .send(Message::Text(format!("connected:{app_name}").into()))
                .await;

            while let Some(Ok(message)) = socket.recv().await {
                match message {
                    Message::Text(text) => {
                        let _ = socket
                            .send(Message::Text(format!("echo:{text}").into()))
                            .await;
                    }
                    Message::Binary(bytes) => {
                        let _ = socket.send(Message::Binary(bytes)).await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        })
    }
}
"#
    .to_string()
}

impl GreeterGrpcService {
    pub fn new(ctx: GrpcContext) -> Self {
        Self { ctx }
    }
}

#[nestforge::tonic::async_trait]
impl Greeter for GreeterGrpcService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name.trim().to_string();
        if name.is_empty() {
            return Err(Status::invalid_argument("name is required"));
        }

        let config = self.ctx.resolve::<AppConfig>()?;
        Ok(Response::new(HelloReply {
            message: format!("Hello, {name}! Welcome to {}.", config.app_name),
        }))
    }
}
"#
    .to_string()
}

fn template_named_grpc_service_rs(service_name: &str, pascal_name: &str) -> String {
    let rpc_name = format!("get_{}_status", service_name);
    format!(
        r#"use nestforge::{{
    tonic::{{Request, Response, Status}},
    GrpcContext,
}};

use crate::grpc::proto::{service_name}::{{
    {pascal_name}StatusReply,
    {pascal_name}StatusRequest,
    {service_name}_service_server::{pascal_name}Service,
}};

#[derive(Clone)]
pub struct {pascal_name}GrpcService {{
    ctx: GrpcContext,
}}

impl {pascal_name}GrpcService {{
    pub fn new(ctx: GrpcContext) -> Self {{
        Self {{ ctx }}
    }}
}}

#[nestforge::tonic::async_trait]
impl {pascal_name}Service for {pascal_name}GrpcService {{
    async fn {rpc_name}(
        &self,
        request: Request<{pascal_name}StatusRequest>,
    ) -> Result<Response<{pascal_name}StatusReply>, Status> {{
        let name = request.into_inner().name.trim().to_string();
        let message = if name.is_empty() {{
            format!("{pascal_name} service is ready")
        }} else {{
            format!("{pascal_name} service is ready for {{name}}")
        }};

        let _ = self.ctx.container();

        Ok(Response::new({pascal_name}StatusReply {{ message }}))
    }}
}}
"#
    )
}

fn template_env_file(app_name: &str, transport: AppTransport) -> String {
    let transport_note = match transport {
        AppTransport::Http => "# Generated for a NestForge HTTP app.\n",
        AppTransport::Graphql => "# Generated for a NestForge GraphQL app.\n",
        AppTransport::Grpc => "# Generated for a NestForge gRPC app.\n",
        AppTransport::Websockets => "# Generated for a NestForge WebSocket app.\n",
    };

    format!(
        "{transport_note}APP_NAME={}\n# Optional when you add SQL migrations later.\nDATABASE_URL=postgres://<user>:<password>@localhost/<database>\n",
        to_pascal_case(app_name).replace('_', " ")
    )
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
}}
"#
    )
}

fn template_resource_service_rs(
    _resource: &str,
    _singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    namespace: &str,
) -> String {
    format!(
        r#"use nestforge::ResourceService;

use {namespace}dto::{pascal_singular}Dto;

pub type {pascal_plural}Service = ResourceService<{pascal_singular}Dto>;
"#
    )
}

fn template_resource_controller_rs(
    resource: &str,
    _singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    namespace: &str,
) -> String {
    format!(
        r#"use axum::Json;
use nestforge::{{controller, routes, ApiResult, Inject, List, OptionHttpExt, Param, ResultHttpExt, ValidatedBody}};

use {namespace}dto::{{Create{pascal_singular}Dto, Update{pascal_singular}Dto, {pascal_singular}Dto}};
use {namespace}services::{pascal_plural}Service;

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

fn template_guard_rs(pascal_guard: &str) -> String {
    format!(
        r#"nestforge::guard!({pascal_guard});
"#
    )
}

fn template_middleware_rs(pascal_middleware: &str) -> String {
    format!(
        r#"nestforge::middleware!({pascal_middleware}, |req, next| {{
    {{
        println!("{{}} {{}}", req.method(), req.uri().path());
        (next)(req).await
    }}
}});
"#
    )
}

fn template_exception_filter_rs(pascal_filter: &str) -> String {
    format!(
        r#"#[derive(Default)]
pub struct {pascal_filter};

impl nestforge::ExceptionFilter for {pascal_filter} {{
    fn catch(
        &self,
        exception: nestforge::HttpException,
        _ctx: &nestforge::RequestContext,
    ) -> nestforge::HttpException {{
        exception
    }}
}}
"#
    )
}

fn template_interceptor_rs(pascal_interceptor: &str) -> String {
    format!(
        r#"nestforge::interceptor!({pascal_interceptor});
"#
    )
}

fn template_serializer_rs(serializer_name: &str, pascal_serializer: &str) -> String {
    format!(
        r#"#[derive(serde::Serialize)]
pub struct {pascal_serializer}Dto {{
    pub id: u64,
    pub label: String,
}}

pub struct {pascal_serializer};

impl nestforge::ResponseSerializer<serde_json::Value> for {pascal_serializer} {{
    type Output = {pascal_serializer}Dto;

    fn serialize(value: serde_json::Value) -> Self::Output {{
        {pascal_serializer}Dto {{
            id: value.get("id").and_then(|value| value.as_u64()).unwrap_or_default(),
            label: value
                .get("label")
                .and_then(|value| value.as_str())
                .unwrap_or("{serializer_name}")
                .to_string(),
        }}
    }}
}}
"#
    )
}

fn template_named_ws_gateway_rs(pascal_gateway: &str) -> String {
    format!(
        r#"use nestforge::{{Message, WebSocket, WebSocketContext, WebSocketGateway}};

pub struct {pascal_gateway};

impl WebSocketGateway for {pascal_gateway} {{
    fn on_connect(
        &self,
        _ctx: WebSocketContext,
        mut socket: WebSocket,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {{
        Box::pin(async move {{
            let _ = socket
                .send(Message::Text("connected".to_string().into()))
                .await;
        }})
    }}
}}
"#
    )
}

fn template_microservice_patterns_rs(pattern_name: &str, pascal_name: &str) -> String {
    format!(
        r#"use nestforge::{{MicroserviceRegistry, TransportMetadata}};

pub struct {pascal_name}Patterns;

impl {pascal_name}Patterns {{
    pub fn registry() -> MicroserviceRegistry {{
        MicroserviceRegistry::builder()
            .message("{pattern_name}.ping", |_payload: (), ctx| async move {{
                Ok(serde_json::json!({{
                    "pattern": ctx.pattern(),
                    "transport": ctx.transport(),
                }}))
            }})
            .event("{pattern_name}.created", |_payload: serde_json::Value, _ctx| async move {{
                Ok(())
            }})
            .build()
    }}

    pub fn metadata() -> TransportMetadata {{
        TransportMetadata::new().insert("module", "{pattern_name}")
    }}
}}
"#
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

fn parse_target_module_arg(args: &[String]) -> Result<Option<String>> {
    if args.is_empty() {
        return Ok(None);
    }

    if args.len() == 2 && args[0] == "--module" {
        let module = normalize_resource_name(&args[1]);
        if module.is_empty() {
            bail!("Module name cannot be empty.");
        }
        return Ok(Some(module));
    }

    bail!("Invalid generator options. Use: --module <feature>");
}

fn generator_target_root(app_root: &Path, target_module: Option<&str>) -> Result<PathBuf> {
    if let Some(module_name) = target_module {
        let root = app_root.join("src").join(module_name);
        if !root.exists() {
            bail!(
                "Target module `{}` not found. Create it first with: nestforge g module {}",
                module_name,
                module_name
            );
        }
        return Ok(root);
    }

    Ok(app_root.join("src"))
}

fn generator_namespace(target_module: Option<&str>) -> String {
    if let Some(module_name) = target_module {
        return format!("crate::{}::", module_name);
    }
    "crate::".to_string()
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

fn template_feature_mod_rs(module_name: &str, pascal_module: &str) -> String {
    format!(
        r#"pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

use self::controllers::{{
    Controller,
    /* nestforge:feature_controllers_use */
}};
use self::services::{{
    Service,
    /* nestforge:feature_services_use */
}};

#[module(
    imports = [],
    controllers = [
        Controller,
        /* nestforge:feature_controllers */
    ],
    providers = [
        Service::new(),
        /* nestforge:feature_providers */
    ],
    exports = [
        Service,
        /* nestforge:feature_exports */
    ]
)]
pub struct {pascal_module};

// Feature module: {module_name}
"#
    )
}

fn template_feature_controllers_mod_rs(_module_name: &str, _pascal_module: &str) -> String {
    "pub mod controller;\n\npub use controller::Controller;\n".to_string()
}

fn template_feature_controller_rs(module_name: &str, _pascal_module: &str) -> String {
    format!(
        r#"use nestforge::{{controller, routes, Inject}};

use crate::{module_name}::services::Service;

#[controller("/{module_name}")]
pub struct Controller;

#[routes]
impl Controller {{
    #[nestforge::get("/")]
    async fn index(service: Inject<Service>) -> String {{
        service.hello()
    }}
}}
"#
    )
}

fn template_feature_services_mod_rs(_module_name: &str, _pascal_module: &str) -> String {
    "pub mod service;\n\npub use service::Service;\n".to_string()
}

fn template_feature_service_rs(module_name: &str, _pascal_module: &str) -> String {
    format!(
        r#"#[derive(Clone)]
pub struct Service;

impl Service {{
    pub fn new() -> Self {{
        Self
    }}

    pub fn hello(&self) -> String {{
        "Hello from {module_name} module".to_string()
    }}
}}
"#
    )
}

fn template_feature_dto_mod_rs() -> String {
    "/* feature dto exports */\n".to_string()
}

fn patch_root_app_module_import(
    app_root: &Path,
    module_name: &str,
    pascal_module: &str,
) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;
    let import_line = format!("use crate::{}::{};\n", module_name, pascal_module);

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
    let decl = format!("mod {};", module_name);
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

fn compute_content_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn contains_sql_content(sql: &str) -> bool {
    let mut chars = sql.chars().peekable();
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if ch == '-' && chars.peek() == Some(&'-') {
            let _ = chars.next();
            in_line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_block_comment = true;
            continue;
        }

        if !ch.is_whitespace() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{
        compute_content_hash, contains_sql_content, parse_new_transport_arg,
        template_microservice_patterns_rs, template_serializer_rs, AppTransport,
    };

    #[test]
    fn contains_sql_content_ignores_comment_only_input() {
        let sql = "-- heading\n/* block comment */\n";

        assert!(!contains_sql_content(sql));
    }

    #[test]
    fn contains_sql_content_detects_real_sql_after_comments() {
        let sql = "-- create users table\nCREATE TABLE users (id INT);\n";

        assert!(contains_sql_content(sql));
    }

    #[test]
    fn compute_content_hash_is_stable() {
        let first = compute_content_hash("select 1;");
        let second = compute_content_hash("select 1;");

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn parse_new_transport_defaults_to_http() {
        let args: Vec<String> = Vec::new();

        assert!(matches!(
            parse_new_transport_arg(&args).expect("transport should parse"),
            AppTransport::Http
        ));
    }

    #[test]
    fn parse_new_transport_accepts_graphql() {
        let args = vec!["--transport".to_string(), "graphql".to_string()];

        assert!(matches!(
            parse_new_transport_arg(&args).expect("transport should parse"),
            AppTransport::Graphql
        ));
    }

    #[test]
    fn parse_new_transport_accepts_microservices() {
        let args = vec!["--transport".to_string(), "microservices".to_string()];

        assert!(matches!(
            parse_new_transport_arg(&args).expect("transport should parse"),
            AppTransport::Microservices
        ));
    }

    #[test]
    fn parse_new_transport_accepts_websockets() {
        let args = vec!["--transport".to_string(), "websockets".to_string()];

        assert!(matches!(
            parse_new_transport_arg(&args).expect("transport should parse"),
            AppTransport::Websockets
        ));
    }

    #[test]
    fn template_microservice_patterns_uses_requested_type_name() {
        let template = template_microservice_patterns_rs("users", "Users");

        assert!(template.contains("pub struct UsersPatterns;"));
        assert!(template.contains(".message(\"users.ping\""));
        assert!(template.contains(".event(\"users.created\""));
    }

    #[test]
    fn template_serializer_uses_requested_type_name() {
        let template = template_serializer_rs("user", "UserSerializer");

        assert!(template.contains("pub struct UserSerializerDto"));
        assert!(template.contains("pub struct UserSerializer;"));
        assert!(template.contains("impl nestforge::ResponseSerializer<serde_json::Value> for UserSerializer"));
    }

    #[test]
    fn template_named_ws_gateway_uses_requested_type_name() {
        let template = template_named_ws_gateway_rs("EventsGateway");

        assert!(template.contains("pub struct EventsGateway;"));
        assert!(template.contains("impl WebSocketGateway for EventsGateway"));
    }

    #[test]
    fn template_exception_filter_uses_requested_type_name() {
        let template = template_exception_filter_rs("RewriteBadRequestFilter");

        assert!(template.contains("pub struct RewriteBadRequestFilter;"));
        assert!(template.contains("impl nestforge::ExceptionFilter for RewriteBadRequestFilter"));
    }
}
