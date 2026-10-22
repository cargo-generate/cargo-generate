// The classic UI symbols carry `#[deprecated]` markers so downstream users
// see them, but internal callers here are just wiring the active backend.
#![allow(deprecated)]

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

    print_ui_next_tip(&args);

    if args.list_favorites {
        list_favorites(&args)?;
    } else if generate(args).is_err() {
        // error already displayed by the ui backend
        std::process::exit(1);
    }

    Ok(())
}

/// Show a one-time tip inviting users to try the upcoming cliclack-based UI.
/// Runs only when the classic backend is active and the invocation looks
/// interactive.
#[cfg(not(feature = "ui-next"))]
fn print_ui_next_tip(args: &cargo_generate::GenerateArgs) {
    use std::io::IsTerminal;

    if args.silent
        || args.list_favorites
        || std::env::var_os("CARGO_GENERATE_NO_TIP").is_some()
        || std::env::var_os("NO_COLOR").is_some()
        || !std::io::stderr().is_terminal()
    {
        return;
    }

    eprintln!(
        "\x1b[33m💡 tip:\x1b[0m a new interactive UI is coming. Try it early:\n     \
         cargo install --features=ui-next --locked cargo-generate\n"
    );
}

#[cfg(feature = "ui-next")]
fn print_ui_next_tip(_args: &cargo_generate::GenerateArgs) {}

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
    if args.template_path.test {
        args.verbose = true;
    };

    args.other_args = Some(other_args);
    if args.template_path.test {
        if args.template_path.auto_path.is_none() {
            args.template_path.path = Some(String::from("."));
        }
        // If auto_path is set, leave it alone — try_from_args_and_config's
        // resolver (classify) decides whether it's a local dir or a remote
        // spec. This stops --test from hijacking github shorthands away from
        // the resolver.
        if args.name.is_none() {
            args.name = names::Generator::default().next();
        }
    }
    args
}
