use anyhow::{bail, Context, Result};
use clap::Parser;
use nestforge_db::{Db, DbConfig};
use owo_colors::OwoColorize;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

mod cli;
mod diagnostics;
mod tui;
mod transpiler;
mod ui;

use crate::cli::{
    AppTransport, Cli, Commands, DbArgs, DbCommand, DevArgs, DocsArgs, DocsFormatArg,
    GenerateArgs, GeneratorKindArg, GeneratorLayout, NewArgs, StartArgs,
};
use crate::diagnostics::{
    app_root_not_found, missing_app_module_declaration, module_file_not_found,
    openapi_feature_missing, render_cli_error,
};
use crate::tui::{
    render_docs_plaintext, run_docs_browser, run_generate_wizard, run_new_wizard,
    should_fallback_to_prompt,
};
use crate::transpiler::transpile_project;
use crate::ui::{
    interactive_enabled, print_brand_banner, print_note, print_success, prompt_generator_kind,
    prompt_transport, start_spinner,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratorOptions {
    target_module: Option<String>,
    layout: GeneratorLayout,
    prompt_for_dto: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DtoFieldSpec {
    name: String,
    ty: DtoFieldType,
    required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DtoFieldType {
    String,
    Bool,
    U32,
    U64,
    I64,
    F64,
}

impl DtoFieldType {
    fn rust_type(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Bool => "bool",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::I64 => "i64",
            Self::F64 => "f64",
        }
    }

    fn prompt_label(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Bool => "bool",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::I64 => "i64",
            Self::F64 => "f64",
        }
    }

    fn choices() -> &'static [Self] {
        &[
            Self::String,
            Self::Bool,
            Self::U32,
            Self::U64,
            Self::I64,
            Self::F64,
        ]
    }
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    if cli.command.is_none() {
        print_brand_banner();
        crate::cli::print_help(&cli)?;
        return Ok(());
    }

    run_cli(cli).map_err(render_cli_error)
}

fn run_cli(cli: Cli) -> Result<()> {
    match cli.command.expect("command checked above") {
        Commands::New(args) => {
            let (app_name, transport, enable_openapi) = resolve_new_args(args)?;
            create_new_app(&app_name, transport, enable_openapi)?;
        }
        Commands::Docs(args) => run_docs_command(args)?,
        Commands::Generate(args) => {
            let (kind, name, options) = resolve_generate_args(args)?;
            run_generate_command(kind, &name, options)?;
        }
        Commands::Db(args) => run_db_command_structured(args)?,
        Commands::ExportDocs(args) => run_export_docs_command_with_options(ExportDocsOptions {
            format: match args.format {
                DocsFormatArg::Json => "json".to_string(),
                DocsFormatArg::Yaml => "yaml".to_string(),
            },
            output: args.output,
            title: args.title,
            version: args.version,
            module_type: args.module_type,
        })?,
        Commands::Fmt => run_fmt_command()?,
        Commands::Start(args) => run_start_command(args)?,
        Commands::Dev(args) => run_dev_command(args)?,
    }

    Ok(())
}

fn run_docs_command(args: DocsArgs) -> Result<()> {
    let interactive = interactive_enabled(true);
    let use_tui = interactive && !args.no_tui;

    if use_tui {
        match run_docs_browser(args.topic.as_deref()) {
            Ok(()) => return Ok(()),
            Err(error) if should_fallback_to_prompt(&error) => {
                print_note(
                    "Full-screen docs browser is unavailable in this terminal. Falling back to plain text.",
                );
            }
            Err(error) => return Err(error),
        }
    }

    println!("{}", render_docs_plaintext(args.topic.as_deref()));
    Ok(())
}

fn resolve_new_args(args: NewArgs) -> Result<(String, AppTransport, bool)> {
    let interactive = interactive_enabled(true);
    let tui_enabled = interactive && !args.no_tui;

    if tui_enabled && (args.app_name.is_none() || args.transport.is_none()) {
        match run_new_wizard() {
            Ok((app_name, transport)) => return Ok((app_name, transport, args.openapi)),
            Err(error) if should_fallback_to_prompt(&error) => {
                print_note(
                    "Full-screen TUI is unavailable in this terminal. Falling back to prompt mode.",
                );
            }
            Err(error) => return Err(error),
        }
    }

    let app_name = match args.app_name {
        Some(name) => name,
        None if interactive => prompt_string("Application name (example: care-api)", false)?,
        None => {
            bail!("Missing app name. Use `nestforge new <app-name>` or run in interactive mode.")
        }
    };

    let transport = match args.transport {
        Some(transport) => transport.into(),
        None if interactive => prompt_transport()?,
        None => AppTransport::Http,
    };

    if args.openapi && !transport_supports_openapi(transport) {
        bail!("OpenAPI scaffolding is currently supported for HTTP and GraphQL apps only.");
    }

    Ok((app_name, transport, args.openapi))
}

fn resolve_generate_args(
    args: GenerateArgs,
) -> Result<(GeneratorKindArg, String, GeneratorOptions)> {
    let interactive = interactive_enabled(true);
    let tui_enabled = interactive && !args.no_tui;

    if tui_enabled && (args.kind.is_none() || args.name.is_none()) {
        match run_generate_wizard() {
            Ok(result) => {
                return Ok((
                    result.kind,
                    result.name,
                    GeneratorOptions {
                        target_module: result.module.map(|value| normalize_resource_name(&value)),
                        layout: result.layout,
                        prompt_for_dto: !result.no_prompt,
                    },
                ));
            }
            Err(error) if should_fallback_to_prompt(&error) => {
                print_note(
                    "Full-screen TUI is unavailable in this terminal. Falling back to prompt mode.",
                );
            }
            Err(error) => return Err(error),
        }
    }

    let kind = match args.kind {
        Some(kind) => kind,
        None if interactive => prompt_generator_kind()?,
        None => bail!("Missing generator kind. Use `nestforge generate <kind> <name>`."),
    };

    let name = match args.name {
        Some(name) => name,
        None if interactive => prompt_string("Resource or module name (example: users)", false)?,
        None => bail!(
            "Missing generator name. Use `nestforge generate {}` <name>.",
            kind.label()
        ),
    };

    let target_module = if args.module.is_some() {
        args.module.map(|value| normalize_resource_name(&value))
    } else if interactive
        && matches!(
            kind,
            GeneratorKindArg::Resource | GeneratorKindArg::Controller | GeneratorKindArg::Service
        )
        && prompt_yes_no("Generate inside a feature module?", false)?
    {
        let module_name = prompt_string("Target module name", false)?;
        Some(normalize_resource_name(&module_name))
    } else {
        None
    };

    Ok((
        kind,
        name,
        GeneratorOptions {
            target_module,
            layout: if args.flat {
                GeneratorLayout::Flat
            } else {
                GeneratorLayout::Nested
            },
            prompt_for_dto: !args.no_prompt,
        },
    ))
}

fn run_generate_command(
    kind: GeneratorKindArg,
    name: &str,
    options: GeneratorOptions,
) -> Result<()> {
    match kind {
        GeneratorKindArg::Resource => generate_resource(
            name,
            options.target_module.as_deref(),
            options.layout,
            options.prompt_for_dto,
        ),
        GeneratorKindArg::Controller => {
            generate_controller_only(name, options.target_module.as_deref(), options.layout)
        }
        GeneratorKindArg::Service => {
            generate_service_only(name, options.target_module.as_deref(), options.layout)
        }
        GeneratorKindArg::Module => generate_module(name, options.layout),
        GeneratorKindArg::Guard => generate_guard_only(name),
        GeneratorKindArg::Decorator => generate_request_decorator_only(name),
        GeneratorKindArg::Filter => generate_exception_filter_only(name),
        GeneratorKindArg::Middleware => generate_middleware_only(name),
        GeneratorKindArg::Interceptor => generate_interceptor_only(name),
        GeneratorKindArg::Serializer => generate_serializer_only(name),
        GeneratorKindArg::Graphql => generate_graphql_resolver_only(name),
        GeneratorKindArg::Grpc => generate_grpc_service_only(name),
        GeneratorKindArg::Gateway => generate_websocket_gateway_only(name),
        GeneratorKindArg::Microservice => generate_microservice_patterns_only(name),
    }
}

fn run_db_command_structured(args: DbArgs) -> Result<()> {
    let app_root = detect_app_root()?;
    match args.action {
        DbCommand::Init => db_init(&app_root),
        DbCommand::Generate { name } => db_generate(&app_root, &name),
        DbCommand::Migrate => db_migrate(&app_root),
        DbCommand::Status => db_status(&app_root),
    }
}

/* ------------------------------
   NEW APP SCAFFOLD
------------------------------ */

fn create_new_app(app_name: &str, transport: AppTransport, enable_openapi: bool) -> Result<()> {
    let app_dir = env::current_dir()?.join(app_name);

    if app_dir.exists() {
        bail!("App `{}` already exists at {}", app_name, app_dir.display());
    }

    let spinner = start_spinner(format!(
        "Scaffolding {} app {}",
        transport.label(),
        app_name.bold()
    ));

    /* Cargo.toml */
    write_file(
        &app_dir.join("Cargo.toml"),
        &template_app_cargo_toml(
            app_name,
            resolve_nestforge_dependency_line(transport, enable_openapi),
            transport,
        ),
    )?;

    /* main.rs */
    write_file(
        &app_dir.join("src/main.rs"),
        &template_main_rs(app_name, transport, enable_openapi),
    )?;
    write_file(
        &app_dir.join("src/lib.rs"),
        &template_app_lib_rs(transport),
    )?;

    /* app_module.rs */
    write_file(
        &app_dir.join("src/app_module.rs"),
        &template_app_module_rs(transport),
    )?;

    write_file(
        &app_dir.join("src/app_config.rs"),
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

    spinner.finish_and_clear();
    print_success(format!(
        "Created NestForge {} app at {}",
        transport.label(),
        app_dir.display()
    ));
    print_note(format!("Next: cd {}", app_dir.display()));
    print_note("Then run: cargo run");

    if enable_openapi {
        print_note("OpenAPI docs will be available at /api/v1/docs for supported HTTP routes.");
    }

    if matches!(transport, AppTransport::Http) {
        print_note("Then generate your first resource: nestforge generate resource users");
    }

    Ok(())
}

fn scaffold_transport_files(app_dir: &Path, transport: AppTransport) -> Result<()> {
    match transport {
        AppTransport::Http => {
            fs::create_dir_all(app_dir.join("src/guards"))?;
            fs::create_dir_all(app_dir.join("src/interceptors"))?;

            write_file(
                &app_dir.join("src/app_service.rs"),
                &template_app_service_rs(),
            )?;
            write_file(
                &app_dir.join("src/app_controller.rs"),
                &template_app_controller_rs(),
            )?;
            write_file(
                &app_dir.join("src/health_controller.rs"),
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
            write_file(&app_dir.join("proto/greeter.proto"), &template_grpc_proto())?;
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

fn run_start_command(args: StartArgs) -> Result<()> {
    let app_root = if let Some(name) = &args.app_name {
        if name == "." {
            detect_app_root().or_else(|_| env::current_dir())?
        } else {
            PathBuf::from(name)
        }
    } else {
        detect_app_root().or_else(|_| env::current_dir())?
    };

    let source_dir = app_root.join("src");
    let cache_dir = app_root.join(".nestforge").join("cache");

    println!("Transpiling TypeScript-style imports...");
    transpile_project(&source_dir, &cache_dir)?;
    println!("Transpilation complete. Running application...");

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    if !args.args.is_empty() {
        cmd.arg("--").args(&args.args);
    }
    
    let status = cmd.current_dir(&app_root).status()
        .with_context(|| format!("Failed to run application in {}", app_root.display()))?;

    if !status.success() {
        bail!("Application failed to start");
    }

    Ok(())
}

fn run_dev_command(args: DevArgs) -> Result<()> {
    let app_root = if let Some(name) = &args.app_name {
        if name == "." {
            detect_app_root().or_else(|_| env::current_dir())?
        } else {
            PathBuf::from(name)
        }
    } else {
        detect_app_root().or_else(|_| env::current_dir())?
    };

    let source_dir = app_root.join("src");
    let cache_dir = app_root.join(".nestforge").join("cache");

    println!("Transpiling TypeScript-style imports...");
    transpile_project(&source_dir, &cache_dir)?;
    println!("Transpilation complete. Starting development server with watch...");

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    if !args.args.is_empty() {
        cmd.arg("--").args(&args.args);
    }
    
    let status = cmd.current_dir(&app_root).status()
        .with_context(|| format!("Failed to run application in {}", app_root.display()))?;

    if !status.success() {
        bail!("Application failed to start");
    }

    Ok(())
}

fn run_export_docs_command_with_options(options: ExportDocsOptions) -> Result<()> {
    let app_root = detect_app_root().or_else(|_| env::current_dir())?;
    run_export_docs_command_with_options_at(app_root, options)
}

fn run_export_docs_command_with_options_at(
    app_root: PathBuf,
    options: ExportDocsOptions,
) -> Result<()> {
    let output = options.output.unwrap_or_else(|| {
        let file_name = match options.format.as_str() {
            "yaml" => "openapi.yaml",
            _ => "openapi.json",
        };
        app_root.join("docs").join(file_name)
    });

    export_openapi_docs(
        &app_root,
        &options.title,
        &options.version,
        &options.module_type,
        &output,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExportDocsOptions {
    format: String,
    output: Option<PathBuf>,
    title: String,
    version: String,
    module_type: String,
}

#[cfg_attr(not(test), allow(dead_code))]
fn parse_export_docs_options(args: &[String]) -> Result<ExportDocsOptions> {
    let mut format = "json".to_string();
    let mut output = None;
    let mut title = "NestForge API".to_string();
    let mut version = "0.1.0".to_string();
    let mut module_type = "AppModule".to_string();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("Use: nestforge export-docs [--format json|yaml] [--output <path>] [--title <title>] [--version <version>] [--module-type <type>]");
                };
                let normalized = value.to_ascii_lowercase();
                if normalized != "json" && normalized != "yaml" {
                    bail!("Unsupported docs format `{value}`. Use `json` or `yaml`.");
                }
                format = normalized;
                index += 2;
            }
            "--output" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("Use: nestforge export-docs [--format json|yaml] [--output <path>] [--title <title>] [--version <version>] [--module-type <type>]");
                };
                output = Some(PathBuf::from(value));
                index += 2;
            }
            "--title" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("Use: nestforge export-docs [--format json|yaml] [--output <path>] [--title <title>] [--version <version>] [--module-type <type>]");
                };
                title = value.clone();
                index += 2;
            }
            "--version" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("Use: nestforge export-docs [--format json|yaml] [--output <path>] [--title <title>] [--version <version>] [--module-type <type>]");
                };
                version = value.clone();
                index += 2;
            }
            "--module-type" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("Use: nestforge export-docs [--format json|yaml] [--output <path>] [--title <title>] [--version <version>] [--module-type <type>]");
                };
                module_type = value.clone();
                index += 2;
            }
            _ => bail!("Use: nestforge export-docs [--format json|yaml] [--output <path>] [--title <title>] [--version <version>] [--module-type <type>]"),
        }
    }

    Ok(ExportDocsOptions {
        format,
        output,
        title,
        version,
        module_type,
    })
}

fn export_openapi_docs(
    app_root: &Path,
    title: &str,
    version: &str,
    module_type: &str,
    output: &Path,
) -> Result<()> {
    let spinner = start_spinner(format!(
        "Exporting OpenAPI {} to {}",
        title.bold(),
        output.display()
    ));
    let main_rs = app_root.join("src/main.rs");
    let temp_bin = app_root.join("src/bin/nestforge_export_docs.rs");
    let top_level_mods = collect_top_level_modules(&main_rs)?;
    let module_lines = top_level_mods
        .iter()
        .map(|module_name| {
            let module_path = resolve_top_level_module_path(app_root, module_name)?;
            let relative = relative_path_from(&temp_bin, &module_path)?;
            Ok(format!("#[path = \"{}\"] mod {};", relative, module_name))
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    let output_path = output
        .canonicalize()
        .unwrap_or_else(|_| output.to_path_buf());
    let output_path_string = output_path.to_string_lossy().replace('\\', "\\\\");

    let script = format!(
        r#"{module_lines}

use anyhow::Context;

fn main() -> anyhow::Result<()> {{
    let doc = nestforge::openapi_doc_for_module::<app_module::{module_type}>("{title}", "{version}")
        .context("Failed to collect OpenAPI metadata. Ensure the app depends on nestforge with the `openapi` feature enabled.")?;
    let path = std::path::PathBuf::from(r"{output_path_string}");
    if let Some(parent) = path.parent() {{
        std::fs::create_dir_all(parent)?;
    }}
    let body = if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {{
        doc.to_openapi_yaml()
    }} else {{
        nestforge::serde_json::to_string_pretty(&doc.to_openapi_json())?
    }};
    std::fs::write(&path, body)?;
    println!("Exported OpenAPI spec to {{}}", path.display());
    Ok(())
}}
"#
    );

    write_file(&temp_bin, &script)?;
    let status = Command::new("cargo")
        .args([
            "run",
            "--offline",
            "--quiet",
            "--bin",
            "nestforge_export_docs",
        ])
        .current_dir(app_root)
        .status()
        .with_context(|| format!("Failed to run cargo export in {}", app_root.display()))?;

    let cleanup_result = fs::remove_file(&temp_bin);
    if let Err(error) = cleanup_result {
        spinner.println(format!(
            "{} failed to remove {}: {error}",
            "warning".bright_yellow(),
            temp_bin.display()
        ));
    }

    if !status.success() {
        spinner.finish_and_clear();
        return Err(openapi_feature_missing());
    }

    spinner.finish_and_clear();
    print_success(format!("Exported OpenAPI spec to {}", output.display()));
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

            tx.execute_script(&sql).await.with_context(|| {
                format!("Migration {} failed while executing SQL script", file_name)
            })?;

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

fn generate_resource(
    name: &str,
    target_module: Option<&str>,
    layout: GeneratorLayout,
    prompt_for_dto: bool,
) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);
    let dto_fields = collect_dto_fields(&pascal_singular, prompt_for_dto)?;
    let spinner = start_spinner(format!("Generating resource {}", resource.bold()));

    let target_root = generator_target_root(&app_root, target_module)?;
    let imports = resource_import_paths(target_module, layout, &resource, &singular);

    generate_dto_files(
        &target_root,
        layout,
        &resource,
        &singular,
        &pascal_singular,
        &dto_fields,
    )?;
    generate_service_file(
        &target_root,
        layout,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &imports,
    )?;
    generate_controller_file(
        &target_root,
        layout,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &imports,
    )?;

    patch_dto_mod(&target_root, layout, &singular, &pascal_singular)?;
    patch_services_mod(&target_root, layout, &resource, &pascal_plural)?;
    patch_controllers_mod(&target_root, layout, &resource, &pascal_plural)?;

    if let Some(module_name) = target_module {
        patch_feature_module(&app_root, module_name, layout, &pascal_plural, true, true)?;
    } else {
        if layout == GeneratorLayout::Flat {
            patch_main_mod_decl(&app_root, &format!("{}_controller", resource))?;
            patch_main_mod_decl(&app_root, &format!("{}_service", resource))?;
            patch_main_mod_decl(&app_root, &format!("{}_dto", singular))?;
            patch_main_mod_decl(&app_root, &format!("create_{}_dto", singular))?;
            patch_main_mod_decl(&app_root, &format!("update_{}_dto", singular))?;
        } else {
            patch_main_mod_decl(&app_root, "controllers")?;
            patch_main_mod_decl(&app_root, "services")?;
            patch_main_mod_decl(&app_root, "dto")?;
        }
        patch_app_module(&app_root, layout, &resource, &pascal_plural)?;
    }

    spinner.finish_and_clear();
    print_success(format!("Generated resource `{}`", resource));
    Ok(())
}

fn generate_controller_only(
    name: &str,
    target_module: Option<&str>,
    layout: GeneratorLayout,
) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);
    let target_root = generator_target_root(&app_root, target_module)?;
    let imports = resource_import_paths(target_module, layout, &resource, &singular);

    generate_controller_file(
        &target_root,
        layout,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &imports,
    )?;
    patch_controllers_mod(&target_root, layout, &resource, &pascal_plural)?;
    if let Some(module_name) = target_module {
        patch_feature_module(&app_root, module_name, layout, &pascal_plural, true, false)?;
    } else {
        if layout == GeneratorLayout::Flat {
            patch_main_mod_decl(&app_root, &format!("{}_controller", resource))?;
        } else {
            patch_main_mod_decl(&app_root, "controllers")?;
        }
        patch_app_module_controllers_only(&app_root, layout, &resource, &pascal_plural)?;
    }

    println!("Generated controller `{}`", resource);
    Ok(())
}

fn generate_service_only(
    name: &str,
    target_module: Option<&str>,
    layout: GeneratorLayout,
) -> Result<()> {
    let app_root = detect_app_root()?;
    let resource = normalize_resource_name(name);
    let singular = singular_name(&resource);
    let pascal_plural = to_pascal_case(&resource);
    let pascal_singular = to_pascal_case(&singular);
    let target_root = generator_target_root(&app_root, target_module)?;
    let imports = resource_import_paths(target_module, layout, &resource, &singular);

    generate_service_file(
        &target_root,
        layout,
        &resource,
        &singular,
        &pascal_plural,
        &pascal_singular,
        &imports,
    )?;
    patch_services_mod(&target_root, layout, &resource, &pascal_plural)?;
    if let Some(module_name) = target_module {
        patch_feature_module(&app_root, module_name, layout, &pascal_plural, false, true)?;
    } else {
        if layout == GeneratorLayout::Flat {
            patch_main_mod_decl(&app_root, &format!("{}_service", resource))?;
        } else {
            patch_main_mod_decl(&app_root, "services")?;
        }
        patch_app_module_providers_only(&app_root, layout, &resource, &pascal_plural)?;
    }

    println!("Generated service `{}`", resource);
    Ok(())
}

fn generate_module(name: &str, layout: GeneratorLayout) -> Result<()> {
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
        &template_feature_mod_rs(&module_name, &pascal_module, layout),
    )?;
    if layout == GeneratorLayout::Nested {
        write_file(
            &module_dir.join("controllers/mod.rs"),
            &template_feature_controllers_mod_rs(&module_name, &pascal_module),
        )?;
        write_file(
            &module_dir.join("services/mod.rs"),
            &template_feature_services_mod_rs(&module_name, &pascal_module),
        )?;
        write_file(
            &module_dir.join("dto/mod.rs"),
            &template_feature_dto_mod_rs(),
        )?;
    }

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

fn generate_request_decorator_only(name: &str) -> Result<()> {
    let app_root = detect_app_root()?;
    let decorator_name = normalize_resource_name(name);
    let pascal_decorator = to_pascal_case(&decorator_name);
    let decorator_file = app_root
        .join("src/decorators")
        .join(format!("{}_decorator.rs", decorator_name));

    if decorator_file.exists() {
        println!(
            "Request decorator already exists: {}",
            decorator_file.display()
        );
        return Ok(());
    }

    fs::create_dir_all(app_root.join("src/decorators"))?;
    write_file(
        &decorator_file,
        &template_request_decorator_rs(&pascal_decorator),
    )?;
    patch_decorators_mod(&app_root, &decorator_name, &pascal_decorator)?;
    patch_main_mod_decl(&app_root, "decorators")?;

    println!("Generated request decorator `{}`", decorator_name);
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
        println!(
            "WebSocket gateway already exists: {}",
            gateway_file.display()
        );
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
        println!(
            "GraphQL resolver already exists: {}",
            resolver_file.display()
        );
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
        write_file(
            &proto_path,
            &template_named_grpc_proto(&service_name, &pascal_name),
        )?;
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
    layout: GeneratorLayout,
    resource: &str,
    singular: &str,
    pascal_singular: &str,
    fields: &[DtoFieldSpec],
) -> Result<()> {
    let dto_dir = dto_dir(target_root, layout);

    let entity_file = dto_dir.join(format!("{}_dto.rs", singular));
    let create_file = dto_dir.join(format!("create_{}_dto.rs", singular));
    let update_file = dto_dir.join(format!("update_{}_dto.rs", singular));

    if !entity_file.exists() {
        write_file(
            &entity_file,
            &template_entity_dto_rs(pascal_singular, fields),
        )?;
    }
    if !create_file.exists() {
        write_file(
            &create_file,
            &template_create_dto_rs(pascal_singular, fields),
        )?;
    }
    if !update_file.exists() {
        write_file(
            &update_file,
            &template_update_dto_rs(pascal_singular, fields),
        )?;
    }

    let _ = resource; // kept for future template customization
    Ok(())
}

fn collect_dto_fields(resource_name: &str, prompt_for_dto: bool) -> Result<Vec<DtoFieldSpec>> {
    if !prompt_for_dto || !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Ok(default_dto_fields());
    }

    println!();
    println!("Configure DTO fields for {resource_name}:");
    if !prompt_yes_no("Customize generated DTO fields?", false)? {
        return Ok(default_dto_fields());
    }

    let mut fields = Vec::new();
    loop {
        let name = prompt_string("Field name (leave empty to finish)", true)?;
        if name.is_empty() {
            break;
        }

        let normalized = normalize_resource_name(&name);
        if normalized.is_empty() {
            println!("Field name cannot be empty.");
            continue;
        }

        if fields
            .iter()
            .any(|field: &DtoFieldSpec| field.name == normalized)
        {
            println!("Field `{normalized}` already exists.");
            continue;
        }

        let ty = prompt_field_type()?;
        let required = prompt_yes_no("Required in Create DTO?", ty != DtoFieldType::Bool)?;
        fields.push(DtoFieldSpec {
            name: normalized,
            ty,
            required,
        });
    }

    if fields.is_empty() {
        Ok(default_dto_fields())
    } else {
        Ok(fields)
    }
}

fn default_dto_fields() -> Vec<DtoFieldSpec> {
    vec![DtoFieldSpec {
        name: "name".to_string(),
        ty: DtoFieldType::String,
        required: true,
    }]
}

fn prompt_string(prompt: &str, allow_empty: bool) -> Result<String> {
    loop {
        print!("{prompt}: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim().to_string();
        if allow_empty || !value.is_empty() {
            return Ok(value);
        }
    }
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    loop {
        print!("{prompt} {suffix}: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let normalized = input.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(default);
        }
        match normalized.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Enter `y` or `n`."),
        }
    }
}

fn prompt_field_type() -> Result<DtoFieldType> {
    println!("Select a field type:");
    for (index, ty) in DtoFieldType::choices().iter().enumerate() {
        println!("  {}. {}", index + 1, ty.prompt_label());
    }

    loop {
        let value = prompt_string("Type number", false)?;
        let Ok(choice) = value.parse::<usize>() else {
            println!("Enter a number from the list.");
            continue;
        };

        if let Some(ty) = DtoFieldType::choices()
            .get(choice.saturating_sub(1))
            .copied()
        {
            return Ok(ty);
        }

        println!("Enter a number from the list.");
    }
}

fn generate_service_file(
    target_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    imports: &ResourceImportPaths,
) -> Result<()> {
    let service_path = services_dir(target_root, layout).join(format!("{}_service.rs", resource));
    if service_path.exists() {
        println!("Service already exists: {}", service_path.display());
        return Ok(());
    }

    write_file(
        &service_path,
        &template_resource_service_rs(resource, singular, pascal_plural, pascal_singular, imports),
    )?;
    Ok(())
}

fn generate_controller_file(
    target_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    imports: &ResourceImportPaths,
) -> Result<()> {
    let controller_path =
        controllers_dir(target_root, layout).join(format!("{}_controller.rs", resource));

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
            imports,
        ),
    )?;
    Ok(())
}

/* ------------------------------
   PATCHERS
------------------------------ */

fn patch_dto_mod(
    target_root: &Path,
    layout: GeneratorLayout,
    singular: &str,
    pascal_singular: &str,
) -> Result<()> {
    if layout == GeneratorLayout::Flat {
        let Some(path) = target_mod_file(target_root) else {
            return Ok(());
        };
        let mut content = fs::read_to_string(&path)?;

        for line in [
            format!("pub mod {}_dto;", singular),
            format!("pub mod create_{}_dto;", singular),
            format!("pub mod update_{}_dto;", singular),
        ] {
            if !content.contains(&line) {
                content = content.replacen(
                    "/* nestforge:feature_modules */",
                    &format!("/* nestforge:feature_modules */\n{line}"),
                    1,
                );
            }
        }

        for line in [
            format!("pub use {}_dto::{}Dto;", singular, pascal_singular),
            format!(
                "pub use create_{}_dto::Create{}Dto;",
                singular, pascal_singular
            ),
            format!(
                "pub use update_{}_dto::Update{}Dto;",
                singular, pascal_singular
            ),
        ] {
            if !content.contains(&line) {
                content = content.replacen(
                    "/* nestforge:feature_reexports */",
                    &format!("/* nestforge:feature_reexports */\n{line}"),
                    1,
                );
            }
        }

        fs::write(path, content)?;
        return Ok(());
    }

    let path = target_root.join("dto/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_dto_mod_rs()
    };

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

fn patch_services_mod(
    target_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    pascal_plural: &str,
) -> Result<()> {
    if layout == GeneratorLayout::Flat {
        let Some(path) = target_mod_file(target_root) else {
            return Ok(());
        };
        let mut content = fs::read_to_string(&path)?;

        for line in [
            format!("pub mod {}_service;", resource),
            format!("pub use {}_service::{}Service;", resource, pascal_plural),
        ] {
            let marker = if line.starts_with("pub mod ") {
                "/* nestforge:feature_modules */"
            } else {
                "/* nestforge:feature_reexports */"
            };
            if !content.contains(&line) {
                content = content.replacen(marker, &format!("{marker}\n{line}"), 1);
            }
        }

        fs::write(path, content)?;
        return Ok(());
    }

    let path = target_root.join("services/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_services_mod_rs()
    };

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

fn patch_controllers_mod(
    target_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    pascal_plural: &str,
) -> Result<()> {
    if layout == GeneratorLayout::Flat {
        let Some(path) = target_mod_file(target_root) else {
            return Ok(());
        };
        let mut content = fs::read_to_string(&path)?;

        for line in [
            format!("pub mod {}_controller;", resource),
            format!(
                "pub use {}_controller::{}Controller;",
                resource, pascal_plural
            ),
        ] {
            let marker = if line.starts_with("pub mod ") {
                "/* nestforge:feature_modules */"
            } else {
                "/* nestforge:feature_reexports */"
            };
            if !content.contains(&line) {
                content = content.replacen(marker, &format!("{marker}\n{line}"), 1);
            }
        }

        fs::write(path, content)?;
        return Ok(());
    }

    let path = target_root.join("controllers/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_controllers_mod_rs()
    };

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

fn patch_decorators_mod(
    app_root: &Path,
    decorator_name: &str,
    pascal_decorator: &str,
) -> Result<()> {
    let path = app_root.join("src/decorators/mod.rs");
    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        template_decorators_mod_rs()
    };

    let mod_line = format!("pub mod {}_decorator;", decorator_name);
    let use_line = format!(
        "pub use {}_decorator::{};",
        decorator_name, pascal_decorator
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
    let use_line = format!(
        "pub use {}_resolver::{}Resolver;",
        resolver_name, pascal_name
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
    let rerun_line =
        format!("    println!(\"cargo:rerun-if-changed=proto/{service_name}.proto\");\n");

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

    if !content.contains(&format!(
        "cargo:rerun-if-changed=proto/{service_name}.proto"
    )) {
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

fn patch_app_module(
    app_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    pascal_plural: &str,
) -> Result<()> {
    patch_app_module_controllers_only(app_root, layout, resource, pascal_plural)?;
    patch_app_module_providers_only(app_root, layout, resource, pascal_plural)?;

    let _ = resource;
    Ok(())
}

fn patch_feature_module(
    app_root: &Path,
    module_name: &str,
    layout: GeneratorLayout,
    pascal_plural: &str,
    include_controller: bool,
    include_service: bool,
) -> Result<()> {
    if layout == GeneratorLayout::Flat {
        return patch_feature_module_flat(
            app_root,
            module_name,
            pascal_plural,
            include_controller,
            include_service,
        );
    }

    let path = app_root.join("src").join(module_name).join("mod.rs");
    let mut content = fs::read_to_string(&path)?;

    let controllers_marker = "/* nestforge:feature_controllers */";
    let providers_marker = "/* nestforge:feature_providers */";
    let exports_marker = "/* nestforge:feature_exports */";

    let controller_entry = format!("controllers::{}Controller,", pascal_plural);
    let provider_entry = format!("services::{}Service,", pascal_plural);
    let export_entry = format!("services::{}Service,", pascal_plural);

    let controller_block = format!("{}\n        {}", controllers_marker, controller_entry);
    let provider_block = format!("{}\n        {}", providers_marker, provider_entry);
    let export_block = format!("{}\n        {}", exports_marker, export_entry);

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

fn patch_feature_module_flat(
    app_root: &Path,
    module_name: &str,
    pascal_plural: &str,
    include_controller: bool,
    include_service: bool,
) -> Result<()> {
    let path = app_root.join("src").join(module_name).join("mod.rs");
    let mut content = fs::read_to_string(&path)?;

    if include_controller {
        let controller_entry = format!("{pascal_plural}Controller,");
        let controller_block =
            format!("/* nestforge:feature_controllers */\n        {controller_entry}");
        if !content.contains(&controller_block) {
            content = content.replacen(
                "/* nestforge:feature_controllers */",
                &format!("/* nestforge:feature_controllers */\n        {controller_entry}"),
                1,
            );
        }
    }

    if include_service {
        let provider_entry = format!("{pascal_plural}Service,");
        let provider_block = format!("/* nestforge:feature_providers */\n        {provider_entry}");
        if !content.contains(&provider_block) {
            content = content.replacen(
                "/* nestforge:feature_providers */",
                &format!("/* nestforge:feature_providers */\n        {provider_entry}"),
                1,
            );
        }

        let export_entry = format!("{pascal_plural}Service,");
        let export_block = format!("/* nestforge:feature_exports */\n        {export_entry}");
        if !content.contains(&export_block) {
            content = content.replacen(
                "/* nestforge:feature_exports */",
                &format!("/* nestforge:feature_exports */\n        {export_entry}"),
                1,
            );
        }
    }

    fs::write(path, content)?;
    Ok(())
}

fn patch_app_module_controllers_only(
    app_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    pascal_plural: &str,
) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;

    let marker = "/* nestforge:controllers */";
    let entry = format!("{}Controller,", pascal_plural);

    if content.contains(&entry) {
        return Ok(());
    }

    content = content.replace(marker, &format!("{}\n        {}", marker, entry));
    let import_line = match layout {
        GeneratorLayout::Flat => {
            format!(
                "use crate::{}_controller::{}Controller;\n",
                resource, pascal_plural
            )
        }
        GeneratorLayout::Nested => {
            format!("use crate::controllers::{}Controller;\n", pascal_plural)
        }
    };
    if !content.contains(&import_line) {
        content = format!("{import_line}{content}");
    }
    fs::write(path, content)?;
    Ok(())
}

fn patch_app_module_providers_only(
    app_root: &Path,
    layout: GeneratorLayout,
    resource: &str,
    pascal_plural: &str,
) -> Result<()> {
    let path = app_root.join("src/app_module.rs");
    let mut content = fs::read_to_string(&path)?;

    let marker = "/* nestforge:providers */";
    let entry = format!("{}Service,", pascal_plural);

    if content.contains(&entry) {
        return Ok(());
    }

    content = content.replace(marker, &format!("{}\n        {}", marker, entry));
    let import_line = match layout {
        GeneratorLayout::Flat => {
            format!(
                "use crate::{}_service::{}Service;\n",
                resource, pascal_plural
            )
        }
        GeneratorLayout::Nested => {
            format!("use crate::services::{}Service;\n", pascal_plural)
        }
    };
    if !content.contains(&import_line) {
        content = format!("{import_line}{content}");
    }
    fs::write(path, content)?;
    Ok(())
}

/* ------------------------------
   TEMPLATE HELPERS
------------------------------ */

#[cfg_attr(not(test), allow(dead_code))]
fn parse_new_transport_arg(args: &[String]) -> Result<AppTransport> {
    if args.is_empty() {
        return Ok(AppTransport::Http);
    }

    if args.len() == 2 && args[0] == "--transport" {
        return AppTransport::parse(&args[1]);
    }

    bail!("Invalid new app options. Use: nestforge new <app-name> --transport <http|graphql|grpc|microservices|websockets>")
}

fn transport_supports_openapi(transport: AppTransport) -> bool {
    matches!(transport, AppTransport::Http | AppTransport::Graphql)
}

fn resolve_nestforge_dependency_line(transport: AppTransport, enable_openapi: bool) -> String {
    let framework_version = env!("CARGO_PKG_VERSION");
    let mut features = match transport {
        AppTransport::Http => vec!["\"config\""],
        AppTransport::Graphql => vec!["\"config\"", "\"graphql\""],
        AppTransport::Grpc => vec!["\"config\"", "\"grpc\""],
        AppTransport::Microservices => vec!["\"config\"", "\"microservices\"", "\"testing\""],
        AppTransport::Websockets => vec!["\"config\"", "\"websockets\""],
    };
    if enable_openapi && transport_supports_openapi(transport) {
        features.push("\"openapi\"");
    }
    let features = features.join(", ");
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
            "nestforge-config = \"1\"\naxum = \"0.8\"\ntokio = { version = \"1\", features = [\"full\"] }\nserde = { version = \"1\", features = [\"derive\"] }\nanyhow = \"1\"\n"
        }
        AppTransport::Graphql => {
            "nestforge-config = \"1\"\naxum = \"0.8\"\nasync-graphql = \"7\"\ntokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\n"
        }
        AppTransport::Grpc => {
            "nestforge-config = \"1\"\naxum = \"0.8\"\ntokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\ntonic = { version = \"0.12\", features = [\"transport\"] }\nprost = \"0.13\"\n"
        }
        AppTransport::Microservices => {
            "nestforge-config = \"1\"\naxum = \"0.8\"\ntokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\nserde = { version = \"1\", features = [\"derive\"] }\nserde_json = \"1\"\n"
        }
        AppTransport::Websockets => {
            "nestforge-config = \"1\"\naxum = \"0.8\"\ntokio = { version = \"1\", features = [\"full\"] }\nanyhow = \"1\"\n"
        }
    };

    let build_dependencies = if matches!(transport, AppTransport::Grpc) {
        "\n[build-dependencies]\ntonic-build = \"0.12\"\nprotoc-bin-vendored = \"3\"\n"
    } else {
        ""
    };

    format!(
        r#"[package]
name = "{app_name}"
version = "0.1.0"
edition = "2021"
{package_extra}

[workspace]

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

fn template_main_rs(app_name: &str, transport: AppTransport, enable_openapi: bool) -> String {
    let crate_name = to_snake_case(app_name);

    match transport {
        AppTransport::Http => {
            let openapi_setup = if enable_openapi {
                format!(
                    "        .with_openapi_docs(\"{} API\", \"1.0.0\")?\n",
                    to_pascal_case(app_name).replace('_', " ")
                )
            } else {
                String::new()
            };

            format!(
                r#"use {crate_name}::AppModule;
use nestforge::prelude::*;

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {{
    NestForgeFactory::<AppModule>::create()?
        .with_global_prefix("api"){openapi_setup}
        .with_version("v1")
        .listen(PORT)
        .await
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    bootstrap().await
}}
"#,
                crate_name = crate_name
            )
        }
        AppTransport::Graphql => {
            let openapi_setup = if enable_openapi {
                format!(
                    "\n        .with_openapi_docs(\"{} API\", \"1.0.0\")?",
                    to_pascal_case(app_name).replace('_', " ")
                )
            } else {
                String::new()
            };

            format!(
                r#"use {crate_name}::{{build_schema, AppConfig, AppModule}};
use nestforge::prelude::*;

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {{
    let factory = NestForgeFactory::<AppModule>::create()?;
    let config = factory.container().resolve::<AppConfig>()?;
    let schema = build_schema(config.app_name.clone());

    factory
{openapi_setup}        .with_graphql_config(schema, GraphQlConfig::new("/graphql").with_graphiql("/"))
        .listen(PORT)
        .await
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    bootstrap().await
}}
"#,
                crate_name = crate_name
            )
        }
        AppTransport::Grpc => format!(
            r#"use {crate_name}::{{proto::hello::greeter_server::GreeterServer, AppModule, GreeterGrpcService}};
use nestforge::prelude::*;

const ADDR: &str = "127.0.0.1:50051";

async fn bootstrap() -> anyhow::Result<()> {{
    NestForgeGrpcFactory::<AppModule>::create()?
        .with_addr(ADDR)
        .listen_with(|ctx, addr| async move {{
            nestforge::tonic::transport::Server::builder()
                .add_service(GreeterServer::new(GreeterGrpcService::new(ctx)))
                .serve(addr)
                .await
        }})
        .await
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    bootstrap().await
}}
"#,
            crate_name = crate_name
        ),
        AppTransport::Microservices => format!(
            r#"use {crate_name}::{{AppModule, AppPatterns}};
use nestforge::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
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

    println!("{{}}", serde_json::to_string_pretty(&response)?);
    module.shutdown()?;
    Ok(())
}}
"#,
            crate_name = crate_name
        ),
        AppTransport::Websockets => format!(
            r#"use {crate_name}::{{AppModule, EventsGateway}};
use nestforge::{{prelude::*, NestForgeFactory, NestForgeFactoryWebSocketExt}};

const PORT: u16 = 3000;

async fn bootstrap() -> anyhow::Result<()> {{
    NestForgeFactory::<AppModule>::create()?
        .with_websocket_gateway(EventsGateway)
        .listen(PORT)
        .await
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    bootstrap().await
}}
"#,
            crate_name = crate_name
        ),
    }
}

fn template_app_lib_rs(transport: AppTransport) -> String {
    match transport {
        AppTransport::Http => r#"pub mod app_config;
pub mod app_controller;
pub mod app_service;
pub mod app_module;
pub mod guards;
pub mod health_controller;
pub mod interceptors;
/* nestforge:app_modules */

pub use app_config::AppConfig;
pub use app_controller::AppController;
pub use app_service::AppService;
pub use app_module::AppModule;
pub use health_controller::HealthController;
/* nestforge:app_reexports */
"#
        .to_string(),
        AppTransport::Graphql => r#"pub mod app_config;
pub mod app_module;
pub mod graphql;
/* nestforge:app_modules */

pub use app_config::AppConfig;
pub use app_module::AppModule;
pub use graphql::schema::build_schema;
/* nestforge:app_reexports */
"#
        .to_string(),
        AppTransport::Grpc => r#"pub mod app_config;
pub mod app_module;
pub mod grpc;
/* nestforge:app_modules */

pub use app_config::AppConfig;
pub use app_module::AppModule;
pub use grpc::proto;
pub use grpc::service::GreeterGrpcService;
/* nestforge:app_reexports */
"#
        .to_string(),
        AppTransport::Microservices => r#"pub mod app_config;
pub mod app_module;
pub mod microservices;
/* nestforge:app_modules */

pub use app_config::AppConfig;
pub use app_module::AppModule;
pub use microservices::AppPatterns;
/* nestforge:app_reexports */
"#
        .to_string(),
        AppTransport::Websockets => r#"pub mod app_config;
pub mod app_module;
pub mod ws;
/* nestforge:app_modules */

pub use app_config::AppConfig;
pub use app_module::AppModule;
pub use ws::EventsGateway;
/* nestforge:app_reexports */
"#
        .to_string(),
    }
}

fn template_app_module_rs(transport: AppTransport) -> String {
    match transport {
        AppTransport::Http => r#"use nestforge::prelude::*;

use crate::{
    app_config::{load_config, AppConfig},
    app_controller::AppController,
    app_service::AppService,
    health_controller::HealthController,
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
        AppConfig,
        load_config(),
        AppService,
        /* nestforge:providers */
    ],
    exports = [nestforge::ConfigService, AppService]
)]
pub struct AppModule;
"#
        .to_string(),
        AppTransport::Graphql | AppTransport::Grpc | AppTransport::Websockets => {
            r#"use nestforge::prelude::*;

use crate::app_config::{load_config, AppConfig};

#[module(
    imports = [],
    providers = [AppConfig, load_config()],
    exports = [nestforge::ConfigService]
)]
pub struct AppModule;
"#
            .to_string()
        }
        AppTransport::Microservices => r#"use nestforge::prelude::*;

use crate::{
    app_config::{load_config, AppConfig},
    microservices::AppPatterns,
};

#[module(
    imports = [],
    controllers = [],
    providers = [AppConfig, load_config(), AppPatterns],
    exports = [nestforge::ConfigService, AppPatterns]
)]
pub struct AppModule;
"#
        .to_string(),
    }
}

fn template_controllers_mod_rs() -> String {
    "/* Controller exports get generated here */\n".to_string()
}

fn template_services_mod_rs() -> String {
    "/* Service exports get generated here */\n".to_string()
}

fn template_guards_mod_rs() -> String {
    "/* Guard exports get generated here */\n".to_string()
}

fn template_decorators_mod_rs() -> String {
    "/* Request decorator exports get generated here */\n".to_string()
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
    r#"use nestforge::prelude::*;

use crate::AppService;

#[controller("")]
pub struct AppController;

#[routes]
impl AppController {
    #[nestforge::get("/")]
    async fn root(service: Inject<AppService>) -> Result<String, HttpException> {
        Ok(service.welcome_message())
    }
}
"#
    .to_string()
}

fn template_app_service_rs() -> String {
    r#"use nestforge::prelude::*;
use nestforge::ConfigService;

use crate::app_config::load_config;

#[injectable(factory = build_app_service)]
pub struct AppService {
    app_name: String,
}

fn build_app_service() -> anyhow::Result<AppService> {
    let config = load_config();

    Ok(AppService {
        app_name: config.get_string_or("APP_NAME", "NestForge App"),
    })
}

impl AppService {
    pub fn welcome_message(&self) -> String {
        format!("Welcome to {}", self.app_name)
    }
}
"#
        .to_string()
}

fn template_health_controller_rs() -> String {
    r#"use nestforge::prelude::*;

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

fn template_app_config_rs(_transport: AppTransport) -> String {
    r#"use nestforge::{ConfigModule, ConfigOptions, ConfigService};

pub fn load_config() -> ConfigService {
    ConfigModule::for_root_with_options(ConfigOptions::new().env_file(".env"))
}
"#
        .to_string()
}

fn template_graphql_schema_rs() -> String {
    r#"use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

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
        r#"use async_graphql::Object;

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
    AppConfig,
    proto::hello::{greeter_server::Greeter, HelloReply, HelloRequest},
};

#[derive(Clone)]
pub struct GreeterGrpcService {
    ctx: GrpcContext,
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
    r#"use nestforge::injectable;

use crate::AppConfig;

#[injectable(factory = build_app_patterns)]
pub struct AppPatterns {
    registry: nestforge::MicroserviceRegistry,
}

fn build_app_patterns() -> AppPatterns {
    AppPatterns {
        registry: nestforge::MicroserviceRegistry::builder()
            .message("app.ping", |payload: serde_json::Value, ctx| async move {
                let config = ctx.resolve::<AppConfig>()?;
                Ok(serde_json::json!({
                    "app_name": config.app_name,
                    "received": payload,
                    "transport": ctx.transport(),
                }))
            })
            .build(),
    }
}

impl AppPatterns {
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
    r#"use nestforge::{Message, WebSocket, WebSocketContext, WebSocketGateway, ConfigService};

use crate::app_config::load_config;

pub struct EventsGateway;

impl WebSocketGateway for EventsGateway {
    fn on_connect(
        &self,
        ctx: WebSocketContext,
        mut socket: WebSocket,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            let config = load_config();
            let app_name = config.get_string_or("APP_NAME", "NestForge WebSockets");

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

fn template_named_grpc_service_rs(service_name: &str, pascal_name: &str) -> String {
    let rpc_name = format!("get_{}_status", service_name);
    format!(
        r#"use nestforge::{{
    tonic::{{Request, Response, Status}},
    GrpcContext,
}};

use crate::proto::{service_name}::{{
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
        AppTransport::Microservices => "# Generated for a NestForge Microservices app.\n",
    };

    format!(
        "{transport_note}APP_NAME={}\n# Optional when you add SQL migrations later.\nDATABASE_URL=postgres://<user>:<password>@localhost/<database>\n",
        to_pascal_case(app_name).replace('_', " ")
    )
}

fn template_entity_dto_rs(pascal_singular: &str, fields: &[DtoFieldSpec]) -> String {
    let field_lines = fields
        .iter()
        .map(template_entity_field_line)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"#[nestforge::dto]
pub struct {pascal_singular}Dto {{
    pub id: u64,
{field_lines}
}}

nestforge::impl_identifiable!({pascal_singular}Dto, id);
"#
    )
}

fn template_create_dto_rs(pascal_singular: &str, fields: &[DtoFieldSpec]) -> String {
    let field_lines = fields
        .iter()
        .map(template_create_field_block)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"#[nestforge::dto]
pub struct Create{pascal_singular}Dto {{
{field_lines}
}}
"#
    )
}

fn template_update_dto_rs(pascal_singular: &str, fields: &[DtoFieldSpec]) -> String {
    let field_lines = fields
        .iter()
        .map(template_update_field_line)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"#[nestforge::dto]
pub struct Update{pascal_singular}Dto {{
{field_lines}
}}
"#
    )
}

fn template_entity_field_line(field: &DtoFieldSpec) -> String {
    let ty = if field.required {
        field.ty.rust_type().to_string()
    } else {
        format!("Option<{}>", field.ty.rust_type())
    };

    format!("    pub {}: {},", field.name, ty)
}

fn template_create_field_block(field: &DtoFieldSpec) -> String {
    let mut lines = Vec::new();
    if field.required {
        lines.push("    #[validate(required)]".to_string());
    }
    let ty = if field.required {
        field.ty.rust_type().to_string()
    } else {
        format!("Option<{}>", field.ty.rust_type())
    };
    lines.push(format!("    pub {}: {},", field.name, ty));
    lines.join("\n")
}

fn template_update_field_line(field: &DtoFieldSpec) -> String {
    format!("    pub {}: Option<{}>,", field.name, field.ty.rust_type())
}

fn template_resource_service_rs(
    _resource: &str,
    _singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    imports: &ResourceImportPaths,
) -> String {
    format!(
        r#"use nestforge::{{injectable, ResourceService}};

 use {entity_dto_import}::{pascal_singular}Dto;
 use {create_dto_import}::Create{pascal_singular}Dto;
 use {update_dto_import}::Update{pascal_singular}Dto;

#[injectable]
#[derive(Default)]
pub struct {pascal_plural}Service {{
    store: ResourceService<{pascal_singular}Dto>,
}}

impl {pascal_plural}Service {{
    pub fn list(&self) -> Vec<{pascal_singular}Dto> {{
        self.store.all()
    }}

    pub fn get(&self, id: u64) -> Option<{pascal_singular}Dto> {{
        self.store.get(id)
    }}

    pub fn create(&self, dto: Create{pascal_singular}Dto) -> Result<{pascal_singular}Dto, nestforge::ResourceError> {{
        self.store.create(dto)
    }}

    pub fn update(&self, id: u64, dto: Update{pascal_singular}Dto) -> Result<Option<{pascal_singular}Dto>, nestforge::ResourceError> {{
        self.store.update(id, dto)
    }}
}}
 "#,
        entity_dto_import = imports.entity_dto_import,
        create_dto_import = imports.create_dto_import,
        update_dto_import = imports.update_dto_import
    )
}

fn template_resource_controller_rs(
    resource: &str,
    _singular: &str,
    pascal_plural: &str,
    pascal_singular: &str,
    imports: &ResourceImportPaths,
) -> String {
    format!(
        r#"use axum::Json;
use nestforge::{{controller, routes, ApiResult, Inject, List, OptionHttpExt, Param, ResultHttpExt, ValidatedBody}};

 use {entity_dto_import}::{pascal_singular}Dto;
 use {create_dto_import}::Create{pascal_singular}Dto;
 use {update_dto_import}::Update{pascal_singular}Dto;
 use {service_import}::{pascal_plural}Service;

#[controller("/{resource}")]
pub struct {pascal_plural}Controller;

#[routes]
impl {pascal_plural}Controller {{
    #[nestforge::get("/")]
    async fn list(
        service: Inject<{pascal_plural}Service>,
    ) -> ApiResult<List<{pascal_singular}Dto>> {{
        Ok(Json(service.list()))
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
        pascal_singular = pascal_singular,
        entity_dto_import = imports.entity_dto_import,
        create_dto_import = imports.create_dto_import,
        update_dto_import = imports.update_dto_import,
        service_import = imports.service_import
    )
}

fn template_guard_rs(pascal_guard: &str) -> String {
    format!(
        r#"nestforge::guard!({pascal_guard});
"#
    )
}

fn template_request_decorator_rs(pascal_decorator: &str) -> String {
    format!(
        r#"pub struct {pascal_decorator};

impl nestforge::RequestDecorator for {pascal_decorator} {{
    type Output = String;

    fn extract(
        _ctx: &nestforge::RequestContext,
        parts: &axum::http::request::Parts,
    ) -> Result<Self::Output, nestforge::HttpException> {{
        parts
            .headers
            .get("x-{header_name}")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .ok_or_else(|| nestforge::HttpException::bad_request("Missing x-{header_name}"))
    }}
}}
"#,
        header_name = to_snake_case(&pascal_decorator).replace('_', "-")
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
        r#"#[nestforge::response_dto]
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

    Err(app_root_not_found())
}

fn collect_top_level_modules(main_rs: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(main_rs)
        .with_context(|| format!("Failed to read {}", main_rs.display()))?;
    let mut modules = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(module_name) = trimmed
            .strip_prefix("mod ")
            .and_then(|value| value.strip_suffix(';'))
        {
            modules.push(module_name.trim().to_string());
        }
    }

    if !modules
        .iter()
        .any(|module_name| module_name == "app_module")
    {
        return Err(missing_app_module_declaration(
            &main_rs.display().to_string(),
            content,
        ));
    }

    Ok(modules)
}

fn resolve_top_level_module_path(app_root: &Path, module_name: &str) -> Result<PathBuf> {
    let src_root = app_root.join("src");
    let file = src_root.join(format!("{module_name}.rs"));
    if file.exists() {
        return Ok(file);
    }

    let directory_mod = src_root.join(module_name).join("mod.rs");
    if directory_mod.exists() {
        return Ok(directory_mod);
    }

    Err(module_file_not_found(
        module_name,
        &src_root.display().to_string(),
    ))
}

fn relative_path_from(from_file: &Path, to_file: &Path) -> Result<String> {
    let from_dir = from_file
        .parent()
        .context("Temporary export file has no parent directory")?;
    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to_file.components().collect::<Vec<_>>();
    let common_len = from_components
        .iter()
        .zip(&to_components)
        .take_while(|(left, right)| left == right)
        .count();

    let mut relative = PathBuf::new();
    for _ in common_len..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common_len..] {
        relative.push(component.as_os_str());
    }

    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn normalize_resource_name(name: &str) -> String {
    to_snake_case(name).replace(' ', "_")
}

#[cfg_attr(not(test), allow(dead_code))]
fn parse_generator_options(args: &[String]) -> Result<GeneratorOptions> {
    let mut target_module = None;
    let mut layout = GeneratorLayout::Nested;
    let mut prompt_for_dto = true;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--module" => {
                let Some(value) = args.get(index + 1) else {
                    bail!(
                        "Invalid generator options. Use: --module <feature> [--flat] [--no-prompt]"
                    );
                };
                let module = normalize_resource_name(value);
                if module.is_empty() {
                    bail!("Module name cannot be empty.");
                }
                target_module = Some(module);
                index += 2;
            }
            "--flat" => {
                layout = GeneratorLayout::Flat;
                index += 1;
            }
            "--no-prompt" => {
                prompt_for_dto = false;
                index += 1;
            }
            _ => bail!("Invalid generator options. Use: --module <feature> [--flat] [--no-prompt]"),
        }
    }

    Ok(GeneratorOptions {
        target_module,
        layout,
        prompt_for_dto,
    })
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

fn dto_dir(target_root: &Path, layout: GeneratorLayout) -> PathBuf {
    match layout {
        GeneratorLayout::Nested => target_root.join("dto"),
        GeneratorLayout::Flat => target_root.to_path_buf(),
    }
}

fn services_dir(target_root: &Path, layout: GeneratorLayout) -> PathBuf {
    match layout {
        GeneratorLayout::Nested => target_root.join("services"),
        GeneratorLayout::Flat => target_root.to_path_buf(),
    }
}

fn controllers_dir(target_root: &Path, layout: GeneratorLayout) -> PathBuf {
    match layout {
        GeneratorLayout::Nested => target_root.join("controllers"),
        GeneratorLayout::Flat => target_root.to_path_buf(),
    }
}

fn target_mod_file(target_root: &Path) -> Option<PathBuf> {
    let path = target_root.join("mod.rs");
    path.exists().then_some(path)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResourceImportPaths {
    entity_dto_import: String,
    create_dto_import: String,
    update_dto_import: String,
    service_import: String,
}

fn resource_import_paths(
    target_module: Option<&str>,
    layout: GeneratorLayout,
    resource: &str,
    singular: &str,
) -> ResourceImportPaths {
    match (target_module, layout) {
        (Some(module_name), GeneratorLayout::Nested) => ResourceImportPaths {
            entity_dto_import: format!("crate::{module_name}::dto"),
            create_dto_import: format!("crate::{module_name}::dto"),
            update_dto_import: format!("crate::{module_name}::dto"),
            service_import: format!("crate::{module_name}::services"),
        },
        (Some(module_name), GeneratorLayout::Flat) => ResourceImportPaths {
            entity_dto_import: format!("crate::{module_name}"),
            create_dto_import: format!("crate::{module_name}"),
            update_dto_import: format!("crate::{module_name}"),
            service_import: format!("crate::{module_name}"),
        },
        (None, GeneratorLayout::Nested) => ResourceImportPaths {
            entity_dto_import: "crate::dto".to_string(),
            create_dto_import: "crate::dto".to_string(),
            update_dto_import: "crate::dto".to_string(),
            service_import: "crate::services".to_string(),
        },
        (None, GeneratorLayout::Flat) => ResourceImportPaths {
            entity_dto_import: format!("crate::{}_dto", singular),
            create_dto_import: format!("crate::create_{}_dto", singular),
            update_dto_import: format!("crate::update_{}_dto", singular),
            service_import: format!("crate::{}_service", resource),
        },
    }
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
                std::option::Option::None => String::new(),
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

fn template_feature_mod_rs(
    module_name: &str,
    pascal_module: &str,
    layout: GeneratorLayout,
) -> String {
    match layout {
        GeneratorLayout::Nested => format!(
            r#"pub mod controllers;
pub mod dto;
pub mod services;

use nestforge::module;

#[module(
    imports = [],
    controllers = [
        /* nestforge:feature_controllers */
    ],
    providers = [
        /* nestforge:feature_providers */
    ],
    exports = [
        /* nestforge:feature_exports */
    ]
)]
pub struct {pascal_module};

// Feature module: {module_name}
"#
        ),
        GeneratorLayout::Flat => format!(
            r#"/* nestforge:feature_modules */

/* nestforge:feature_reexports */

use nestforge::module;

#[module(
    imports = [],
    controllers = [
        /* nestforge:feature_controllers */
    ],
    providers = [
        /* nestforge:feature_providers */
    ],
    exports = [
        /* nestforge:feature_exports */
    ]
)]
pub struct {pascal_module};

// Feature module: {module_name}
"#
        ),
    }
}

fn template_feature_controllers_mod_rs(_module_name: &str, _pascal_module: &str) -> String {
    template_controllers_mod_rs()
}

fn template_feature_services_mod_rs(_module_name: &str, _pascal_module: &str) -> String {
    template_services_mod_rs()
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
    let lib_path = app_root.join("src/lib.rs");
    if lib_path.exists() {
        let mut content = fs::read_to_string(&lib_path)?;
        let decl = format!("pub mod {};", module_name);
        let reexport = format!("pub use {}::*;", module_name);

        if !content.contains(&decl) {
            if content.contains("/* nestforge:app_modules */") {
                content = content.replace(
                    "/* nestforge:app_modules */",
                    &format!("/* nestforge:app_modules */\n{decl}"),
                );
            } else {
                content.push_str(&format!("\n{decl}"));
            }
        }

        if !content.contains(&reexport) {
            if content.contains("/* nestforge:app_reexports */") {
                content = content.replace(
                    "/* nestforge:app_reexports */",
                    &format!("/* nestforge:app_reexports */\n{reexport}"),
                );
            } else {
                content.push_str(&format!("\n{reexport}"));
            }
        }

        fs::write(lib_path, content)?;
        return Ok(());
    }

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
    use std::path::PathBuf;

    use super::{
        compute_content_hash, contains_sql_content, parse_export_docs_options,
        parse_generator_options, parse_new_transport_arg, template_create_dto_rs,
        template_exception_filter_rs, template_feature_mod_rs, template_microservice_patterns_rs,
        template_named_ws_gateway_rs, template_request_decorator_rs, template_serializer_rs,
        AppTransport, DtoFieldSpec, DtoFieldType, ExportDocsOptions, GeneratorLayout,
        GeneratorOptions,
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
    fn template_app_cargo_toml_declares_workspace_root() {
        let manifest = super::template_app_cargo_toml(
            "demo-app",
            "nestforge = { version = \"1.2.1\", features = [\"config\"] }".to_string(),
            AppTransport::Http,
        );

        assert!(manifest.contains("\n[workspace]\n"));
    }

    #[test]
    fn resolve_nestforge_dependency_line_adds_openapi_for_http_apps() {
        let dependency = super::resolve_nestforge_dependency_line(AppTransport::Http, true);

        assert!(dependency.contains("\"openapi\""));
    }

    #[test]
    fn template_main_rs_wires_openapi_when_requested() {
        let main_rs = super::template_main_rs("marknon", AppTransport::Http, true);

        assert!(main_rs.contains("use nestforge::prelude::*;"));
        assert!(main_rs.contains(".with_openapi_docs(\"Marknon API\", \"1.0.0\")?"));
    }

    #[test]
    fn template_main_rs_generates_valid_microservices_json_macro() {
        let main_rs = super::template_main_rs("professor", AppTransport::Microservices, false);

        assert!(main_rs.contains("serde_json::json!({"));
        assert!(!main_rs.contains("serde_json::json!({{"));
    }

    #[test]
    fn template_main_rs_declares_root_app_service_for_http_apps() {
        let main_rs = super::template_main_rs("demo-api", AppTransport::Http, false);

        assert!(main_rs.contains("use demo_api::AppModule;"));
        assert!(!main_rs.contains("mod app_service;"));
    }

    #[test]
    fn template_app_lib_rs_reexports_root_http_symbols() {
        let lib_rs = super::template_app_lib_rs(AppTransport::Http);

        assert!(lib_rs.contains("pub mod app_service;"));
        assert!(lib_rs.contains("pub use app_module::AppModule;"));
        assert!(lib_rs.contains("/* nestforge:app_modules */"));
        assert!(lib_rs.contains("/* nestforge:app_reexports */"));
    }

    #[test]
    fn template_graphql_schema_imports_async_graphql_crate_path() {
        let schema = super::template_graphql_schema_rs();

        assert!(schema
            .contains("use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};"));
    }

    #[test]
    fn template_app_cargo_toml_includes_async_graphql_for_graphql_apps() {
        let manifest = super::template_app_cargo_toml(
            "demo-api",
            super::resolve_nestforge_dependency_line(AppTransport::Graphql, false),
            AppTransport::Graphql,
        );

        assert!(manifest.contains("async-graphql = \"7\""));
    }

    #[test]
    fn template_app_module_registers_root_app_service_for_http_apps() {
        let module_rs = super::template_app_module_rs(AppTransport::Http);

        assert!(module_rs.contains("app_service::AppService"));
        assert!(module_rs.contains("providers = ["));
        assert!(module_rs.contains("AppConfig,"));
        assert!(module_rs.contains("AppService,"));
        assert!(module_rs.contains("exports = [nestforge::ConfigService"));
    }

    #[test]
    fn template_app_cargo_toml_includes_axum_for_microservices() {
        let manifest = super::template_app_cargo_toml(
            "demo-app",
            "nestforge = { version = \"1.2.1\", features = [\"config\", \"microservices\"] }"
                .to_string(),
            AppTransport::Microservices,
        );

        assert!(manifest.contains("axum = \"0.8\""));
    }

    #[test]
    fn parse_generator_options_supports_flat_module_generation() {
        let options = parse_generator_options(&[
            "--module".to_string(),
            "users".to_string(),
            "--flat".to_string(),
        ])
        .expect("generator options should parse");

        assert_eq!(
            options,
            GeneratorOptions {
                target_module: Some("users".to_string()),
                layout: GeneratorLayout::Flat,
                prompt_for_dto: true,
            }
        );
    }

    #[test]
    fn parse_generator_options_supports_no_prompt() {
        let options = parse_generator_options(&["--no-prompt".to_string()])
            .expect("generator options should parse");

        assert_eq!(
            options,
            GeneratorOptions {
                target_module: None,
                layout: GeneratorLayout::Nested,
                prompt_for_dto: false,
            }
        );
    }

    #[test]
    fn parse_export_docs_options_supports_custom_output() {
        let options = parse_export_docs_options(&[
            "--format".to_string(),
            "yaml".to_string(),
            "--output".to_string(),
            "docs/openapi.yaml".to_string(),
            "--title".to_string(),
            "Users API".to_string(),
            "--version".to_string(),
            "2.0.0".to_string(),
        ])
        .expect("export options should parse");

        assert_eq!(
            options,
            ExportDocsOptions {
                format: "yaml".to_string(),
                output: Some(PathBuf::from("docs/openapi.yaml")),
                title: "Users API".to_string(),
                version: "2.0.0".to_string(),
                module_type: "AppModule".to_string(),
            }
        );
    }

    #[test]
    fn template_create_dto_uses_custom_field_specs() {
        let template = template_create_dto_rs(
            "User",
            &[
                DtoFieldSpec {
                    name: "email".to_string(),
                    ty: DtoFieldType::String,
                    required: true,
                },
                DtoFieldSpec {
                    name: "age".to_string(),
                    ty: DtoFieldType::U32,
                    required: false,
                },
            ],
        );

        assert!(template.contains("#[validate(required)]"));
        assert!(template.contains("pub email: String,"));
        assert!(template.contains("pub age: Option<u32>,"));
    }

    #[test]
    fn flat_feature_module_template_exposes_root_level_exports() {
        let template = template_feature_mod_rs("users", "UsersModule", GeneratorLayout::Flat);

        assert!(template.contains("/* nestforge:feature_modules */"));
        assert!(template.contains("/* nestforge:feature_reexports */"));
        assert!(!template.contains("pub mod controller;"));
        assert!(!template.contains("pub mod service;"));
        assert!(!template.contains("Controller,"));
        assert!(!template.contains("Service,"));
        assert!(!template.contains("pub mod controllers;"));
    }

    #[test]
    fn nested_feature_module_template_starts_without_placeholder_imports() {
        let template = template_feature_mod_rs("users", "UsersModule", GeneratorLayout::Nested);

        assert!(template.contains("pub mod controllers;"));
        assert!(template.contains("pub mod services;"));
        assert!(template.contains("pub mod dto;"));
        assert!(!template.contains("use self::controllers"));
        assert!(!template.contains("use self::services"));
        assert!(!template.contains("Controller,"));
        assert!(!template.contains("Service,"));
    }

    #[test]
    fn template_microservice_patterns_uses_requested_type_name() {
        let template = template_microservice_patterns_rs("users", "Users");

        assert!(template.contains("pub struct UsersPatterns;"));
        assert!(template.contains(".message(\"users.ping\""));
        assert!(template.contains(".event(\"users.created\""));
    }

    #[test]
    fn template_request_decorator_uses_requested_type_name() {
        let template = template_request_decorator_rs("CorrelationId");

        assert!(template.contains("pub struct CorrelationId;"));
        assert!(template.contains("impl nestforge::RequestDecorator for CorrelationId"));
        assert!(template.contains("x-correlation-id"));
    }

    #[test]
    fn template_serializer_uses_requested_type_name() {
        let template = template_serializer_rs("user", "UserSerializer");

        assert!(template.contains("pub struct UserSerializerDto"));
        assert!(template.contains("pub struct UserSerializer;"));
        assert!(template
            .contains("impl nestforge::ResponseSerializer<serde_json::Value> for UserSerializer"));
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

    #[test]
    fn cli_docs_cover_generation_workflow() {
        let docs = super::render_docs_plaintext(None);

        assert!(docs.contains("nestforge new my-app --transport http --no-tui"));
        assert!(docs.contains("nestforge g module users"));
        assert!(docs.contains("nestforge g resource users --module users"));
        assert!(docs.contains("nestforge export-docs --format yaml"));
        assert!(docs.contains("nestforge db migrate"));
    }
}
