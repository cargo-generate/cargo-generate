use anyhow::Result;
use cargo_generate::{generate, Cli};
use clap::Parser;

fn main() -> Result<()> {
    let Cli::Generate(args) = Cli::parse();
    generate(args)?;

    Ok(())
}
