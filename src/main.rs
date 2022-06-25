use anyhow::Result;
use cargo_generate::{app_config_path, generate, list_favorites, AppConfig, Cli};
use clap::Parser;

fn main() -> Result<()> {
    let Cli::Generate(args) = Cli::parse();
    let app_config: AppConfig = app_config_path(&args.config)?.as_path().try_into()?;

    if args.list_favorites {
        list_favorites(&app_config, &args)?;
    } else {
        generate(app_config, args)?;
    }

    Ok(())
}
