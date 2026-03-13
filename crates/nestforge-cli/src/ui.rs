use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::{
    io::{self, IsTerminal},
};

use crate::cli::{AppTransport, GeneratorKindArg};

pub const BRAND_BANNER: [&str; 6] = [
    "\u{2588}\u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}",
    "\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\u{255a}\u{2550}\u{2550}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d} \u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}",
    "\u{2588}\u{2588}\u{2554}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{2588}\u{2588}\u{2551}  \u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  ",
    "\u{2588}\u{2588}\u{2551}\u{255a}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}  \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}  \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}  ",
    "\u{2588}\u{2588}\u{2551} \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}     \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{2588}\u{2588}\u{2551}  \u{2588}\u{2588}\u{2551}\u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}",
    "\u{255a}\u{2550}\u{255d}  \u{255a}\u{2550}\u{2550}\u{2550}\u{255d}\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}   \u{255a}\u{2550}\u{255d}   \u{255a}\u{2550}\u{255d}      \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d} \u{255a}\u{2550}\u{255d}  \u{255a}\u{2550}\u{255d} \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d} \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}",
];

pub fn print_brand_banner() {
    for line in BRAND_BANNER {
        println!("{}", line.bright_cyan().bold());
    }
    println!("{}", "Scaffold. Generate. Ship.".dimmed());
}

pub fn start_spinner(message: impl Into<String>) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("spinner template should be valid"),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner.set_message(message.into());
    spinner
}

pub fn print_success(message: impl AsRef<str>) {
    println!("{} {}", "[ok]".bright_green(), message.as_ref());
}

pub fn print_note(message: impl AsRef<str>) {
    println!("{} {}", "[>]".bright_blue(), message.as_ref());
}

pub fn interactive_enabled(enabled: bool) -> bool {
    enabled && io::stdin().is_terminal() && io::stdout().is_terminal()
}

pub fn prompt_transport() -> Result<AppTransport> {
    let choices = [
        AppTransport::Http,
        AppTransport::Graphql,
        AppTransport::Grpc,
        AppTransport::Microservices,
        AppTransport::Websockets,
    ];
    println!("Select a transport:");
    for (index, choice) in choices.iter().enumerate() {
        println!("  {}. {}", index + 1, choice);
    }

    loop {
        let value = crate::prompt_string("Transport number", false)?;
        let Ok(choice) = value.parse::<usize>() else {
            println!("Enter a number from the list.");
            continue;
        };
        if let Some(transport) = choices.get(choice.saturating_sub(1)).copied() {
            return Ok(transport);
        }
        println!("Enter a number from the list.");
    }
}

pub fn prompt_generator_kind() -> Result<GeneratorKindArg> {
    let choices = [
        GeneratorKindArg::Resource,
        GeneratorKindArg::Controller,
        GeneratorKindArg::Service,
        GeneratorKindArg::Module,
        GeneratorKindArg::Guard,
        GeneratorKindArg::Decorator,
        GeneratorKindArg::Filter,
        GeneratorKindArg::Middleware,
        GeneratorKindArg::Interceptor,
        GeneratorKindArg::Serializer,
        GeneratorKindArg::Graphql,
        GeneratorKindArg::Grpc,
        GeneratorKindArg::Gateway,
        GeneratorKindArg::Microservice,
    ];
    println!("Select a generator:");
    for (index, choice) in choices.iter().enumerate() {
        println!("  {}. {}", index + 1, choice);
    }

    loop {
        let value = crate::prompt_string("Generator number", false)?;
        let Ok(choice) = value.parse::<usize>() else {
            println!("Enter a number from the list.");
            continue;
        };
        if let Some(kind) = choices.get(choice.saturating_sub(1)).copied() {
            return Ok(kind);
        }
        println!("Enter a number from the list.");
    }
}
