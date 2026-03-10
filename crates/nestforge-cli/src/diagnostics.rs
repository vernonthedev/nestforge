use anyhow::Error;
use miette::{miette, Report};

pub fn render_cli_error(error: Error) -> Report {
    let message = error.to_string();

    if message.contains("Run generator inside an app folder") {
        return miette!(
            help = "Change into a NestForge app directory containing `Cargo.toml` and `src/`, then rerun the command.",
            "{}",
            message
        );
    }

    if message.contains("openapi") && message.contains("feature") {
        return miette!(
            help = "Enable the dependency feature in your app: nestforge = { features = [\"openapi\"] }",
            "{}",
            message
        );
    }

    if message.contains("Target module") && message.contains("not found") {
        return miette!(
            help = "Create the module first with `nestforge generate module <name>` or point `--module` at an existing feature.",
            "{}",
            message
        );
    }

    miette!("{}", message)
}
