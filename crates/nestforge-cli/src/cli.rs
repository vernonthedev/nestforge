use anyhow::{bail, Result};
use clap::{
    builder::{styling::AnsiColor, Styles},
    Args, CommandFactory, Parser, Subcommand, ValueEnum,
};
use miette::IntoDiagnostic;
use std::{fmt, path::PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "nestforge",
    about = "NestForge CLI for scaffolding modular Rust backends",
    long_about = None,
    styles = cli_styles(),
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new NestForge application
    New(NewArgs),
    /// Browse CLI workflow and generator documentation
    Docs(DocsArgs),
    /// Generate framework resources and feature modules
    #[command(alias = "g")]
    Generate(GenerateArgs),
    /// Database migration commands
    Db(DbArgs),
    /// Export OpenAPI documentation
    #[command(visible_alias = "openapi")]
    ExportDocs(ExportDocsArgs),
    /// Format Rust sources with cargo fmt
    Fmt,
}

#[derive(Args, Debug)]
pub struct NewArgs {
    /// Application name
    pub app_name: Option<String>,
    /// Transport runtime
    #[arg(long, value_enum)]
    pub transport: Option<TransportArg>,
    /// Enable OpenAPI docs wiring for supported transports
    #[arg(long)]
    pub openapi: bool,
    /// Disable interactive prompts
    #[arg(long)]
    pub no_tui: bool,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Generator kind
    #[arg(value_enum)]
    pub kind: Option<GeneratorKindArg>,
    /// Resource or module name
    pub name: Option<String>,
    /// Feature module target
    #[arg(long)]
    pub module: Option<String>,
    /// Generate files in the current module root
    #[arg(long)]
    pub flat: bool,
    /// Disable DTO field prompts
    #[arg(long)]
    pub no_prompt: bool,
    /// Disable interactive prompts
    #[arg(long)]
    pub no_tui: bool,
}

#[derive(Args, Debug)]
pub struct DocsArgs {
    /// Optional docs topic
    pub topic: Option<String>,
    /// Render plain text docs instead of the terminal browser
    #[arg(long)]
    pub no_tui: bool,
}

#[derive(Args, Debug)]
pub struct DbArgs {
    #[command(subcommand)]
    pub action: DbCommand,
}

#[derive(Subcommand, Debug)]
pub enum DbCommand {
    Init,
    Generate {
        /// Migration name
        name: String,
    },
    Migrate,
    Status,
}

#[derive(Args, Debug)]
pub struct ExportDocsArgs {
    /// Output format
    #[arg(long, value_enum, default_value_t = DocsFormatArg::Json)]
    pub format: DocsFormatArg,
    /// Output file path
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// OpenAPI title
    #[arg(long, default_value = "NestForge API")]
    pub title: String,
    /// OpenAPI version
    #[arg(long, default_value = "0.1.0")]
    pub version: String,
    /// Root module type
    #[arg(long, default_value = "AppModule")]
    pub module_type: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TransportArg {
    Http,
    Graphql,
    Grpc,
    Microservices,
    Websockets,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum GeneratorKindArg {
    Resource,
    Controller,
    Service,
    Module,
    Guard,
    Decorator,
    Filter,
    Middleware,
    Interceptor,
    Serializer,
    Graphql,
    Grpc,
    Gateway,
    Microservice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum DocsFormatArg {
    Json,
    Yaml,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppTransport {
    Http,
    Graphql,
    Grpc,
    Microservices,
    Websockets,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratorLayout {
    Nested,
    Flat,
}

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightCyan.on_default().bold())
        .usage(AnsiColor::BrightYellow.on_default().bold())
        .literal(AnsiColor::BrightGreen.on_default())
        .placeholder(AnsiColor::BrightMagenta.on_default())
        .error(AnsiColor::BrightRed.on_default().bold())
        .valid(AnsiColor::BrightGreen.on_default().bold())
        .invalid(AnsiColor::BrightRed.on_default().bold())
}

impl AppTransport {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "http" => Ok(Self::Http),
            "graphql" => Ok(Self::Graphql),
            "grpc" => Ok(Self::Grpc),
            "microservices" | "ms" => Ok(Self::Microservices),
            "websockets" | "ws" => Ok(Self::Websockets),
            _ => bail!(
                "Unknown transport `{value}`. Use: http | graphql | grpc | microservices | websockets"
            ),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Http => "HTTP",
            Self::Graphql => "GraphQL",
            Self::Grpc => "gRPC",
            Self::Microservices => "Microservices",
            Self::Websockets => "WebSocket",
        }
    }
}

impl fmt::Display for AppTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl From<TransportArg> for AppTransport {
    fn from(value: TransportArg) -> Self {
        match value {
            TransportArg::Http => Self::Http,
            TransportArg::Graphql => Self::Graphql,
            TransportArg::Grpc => Self::Grpc,
            TransportArg::Microservices => Self::Microservices,
            TransportArg::Websockets => Self::Websockets,
        }
    }
}

impl GeneratorKindArg {
    pub fn label(self) -> &'static str {
        match self {
            Self::Resource => "resource",
            Self::Controller => "controller",
            Self::Service => "service",
            Self::Module => "module",
            Self::Guard => "guard",
            Self::Decorator => "decorator",
            Self::Filter => "filter",
            Self::Middleware => "middleware",
            Self::Interceptor => "interceptor",
            Self::Serializer => "serializer",
            Self::Graphql => "graphql",
            Self::Grpc => "grpc",
            Self::Gateway => "gateway",
            Self::Microservice => "microservice",
        }
    }
}

impl fmt::Display for GeneratorKindArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

pub fn print_help(cli: &Cli) -> miette::Result<()> {
    if cli.command.is_none() {
        Cli::command().print_help().into_diagnostic()?;
        println!();
    }

    Ok(())
}
