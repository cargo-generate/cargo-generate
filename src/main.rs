use anyhow::Result;
use cargo_generate::{generate, Args};
use clap::Parser;
use std::env;

fn main() -> Result<()> {
    let mut args = env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "generate");

    let args = Args::parse_from(command.into_iter().chain(args));
    generate(args)?;

    Ok(())
}
