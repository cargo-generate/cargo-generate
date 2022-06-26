use anyhow::Result;
use cargo_generate::{generate, list_favorites, Cli};
use clap::Parser;

fn main() -> Result<()> {
    let Cli::Generate(args) = Cli::parse();

    if args.list_favorites {
        list_favorites(&args)?;
    } else {
        generate(args)?;
    }

    Ok(())
}
