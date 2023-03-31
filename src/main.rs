use std::path::PathBuf;

use anyhow::Result;
use cargo_generate::{generate, list_favorites, Cli};
use clap::Parser;

fn main() -> Result<()> {
    env_logger::builder()
        .format(cargo_generate::log_formatter)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .format_level(false)
        .target(env_logger::Target::Stdout)
        .init();

    let args = resolve_args();

    if args.list_favorites {
        list_favorites(&args)?;
    } else {
        generate(args)?;
    }

    Ok(())
}

fn resolve_args() -> cargo_generate::GenerateArgs {
    let (args, other_args): (Vec<_>, Vec<_>) = {
        let mut before_other_args = true;
        std::env::args().partition(|a| {
            if before_other_args && a == "--test" {
                before_other_args = false;
                true
            } else {
                before_other_args
            }
        })
    };
    let Cli::Generate(mut args) = Cli::parse_from(args);
    args.other_args = Some(other_args);
    if args.template_path.test {
        args.template_path.path = Some(
            args.template_path
                .auto_path
                .take()
                .map(|sub| PathBuf::from(".").join(sub).display().to_string())
                .unwrap_or_else(|| String::from(".")),
        );
        if args.name.is_none() {
            args.name = names::Generator::default().next();
        }
    }
    args
}
