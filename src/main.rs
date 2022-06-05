use anyhow::Result;
use cargo_generate::{generate, Cli};
use structopt::StructOpt;

fn main() -> Result<()> {
    let Cli::Generate(args) = Cli::from_args();
    generate(args, None)?;

    Ok(())
}
