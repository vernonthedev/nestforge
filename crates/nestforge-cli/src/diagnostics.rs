#![allow(unused_assignments)]

use anyhow::Error;
use miette::{miette, Diagnostic, NamedSource, Report, SourceSpan};
use thiserror::Error;

#[allow(unused_assignments)]
#[derive(Debug, Error, Diagnostic)]
pub enum CliDiagnostic {
    #[error("Run generator inside a NestForge app folder.")]
    #[diagnostic(
        code(nestforge::cli::app_root_not_found),
        help("Change into a NestForge app directory containing `Cargo.toml` and `src/`, then rerun the command.")
    )]
    AppRootNotFound,

    #[error("Could not find `mod app_module;` in {path}.")]
    #[diagnostic(
        code(nestforge::cli::missing_app_module),
        help("Ensure `src/main.rs` declares `mod app_module;` or pass `--module-type` to `export-docs`.")
    )]
    AppModuleDeclarationMissing {
        path: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("`app_module` is not declared in this file")]
        span: SourceSpan,
    },

    #[error("Could not resolve module `{module_name}` under {src_root}.")]
    #[diagnostic(
        code(nestforge::cli::module_file_not_found),
        help("Add `{module_name}.rs` or `{module_name}/mod.rs` under `src/`, or update the app root module declarations.")
    )]
    ModuleFileNotFound {
        module_name: String,
        src_root: String,
    },

    #[error("OpenAPI export failed because the application does not expose the required OpenAPI support.")]
    #[diagnostic(
        code(nestforge::cli::openapi_feature_missing),
        help("Enable `nestforge` with `features = [\"openapi\"]` in the app's `Cargo.toml`, then ensure the app builds cleanly.")
    )]
    OpenApiFeatureMissing,
}

pub fn app_root_not_found() -> Error {
    Error::new(CliDiagnostic::AppRootNotFound)
}

pub fn missing_app_module_declaration(path: &str, src: String) -> Error {
    Error::new(CliDiagnostic::AppModuleDeclarationMissing {
        path: path.to_string(),
        span: SourceSpan::from((0usize, src.len().max(1))),
        src: NamedSource::new(path.to_string(), src),
    })
}

pub fn module_file_not_found(module_name: &str, src_root: &str) -> Error {
    Error::new(CliDiagnostic::ModuleFileNotFound {
        module_name: module_name.to_string(),
        src_root: src_root.to_string(),
    })
}

pub fn openapi_feature_missing() -> Error {
    Error::new(CliDiagnostic::OpenApiFeatureMissing)
}

pub fn render_cli_error(error: Error) -> Report {
    match error.downcast::<CliDiagnostic>() {
        Ok(diagnostic) => return Report::new(diagnostic),
        Err(error) => {
            let message = error.to_string();

            if message.contains("Target module") && message.contains("not found") {
                return miette!(
                    help = "Create the module first with `nestforge generate module <name>` or point `--module` at an existing feature.",
                    "{}",
                    message
                );
            }

            return miette!("{}", message);
        }
    }
}
