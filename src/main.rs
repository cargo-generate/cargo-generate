use anyhow::Result;
use cargo_generate::{generate, list_favorites, Cli};
use clap::Parser;

fn main() -> Result<()> {
    let args = {
        let (args, mut other_args): (Vec<_>, Vec<_>) = {
            let mut before_other_args = true;
            std::env::args().partition(|a| {
                if before_other_args && a == "--" {
                    before_other_args = false;
                }
                before_other_args
            })
        };
        if !other_args.is_empty() {
            other_args.remove(0);
        }

        let Cli::Generate(mut args) = Cli::parse_from(args);
        args.other_args = Some(other_args);
        args
    };

    if args.list_favorites {
        list_favorites(&args)?;
    } else {
        generate(args)?;
    }

    Ok(())
}
