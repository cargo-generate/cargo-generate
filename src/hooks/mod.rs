use anyhow::{Context, Result};
use console::style;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTitleCase, ToUpperCamelCase,
};
use rhai::EvalAltResult;
use std::cell::RefCell;
use std::rc::Rc;
use std::{env, path::Path};

use crate::config;
use crate::emoji;

mod file_mod;
mod system_mod;
mod variable_mod;

type HookResult<T> = std::result::Result<T, Box<EvalAltResult>>;

struct CleanupJob<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> CleanupJob<F> {
    pub fn new(f: F) -> Self {
        Self(Some(f))
    }
}

impl<F: FnOnce()> Drop for CleanupJob<F> {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}

pub fn execute_pre_hooks(
    dir: &Path,
    liquid_object: Rc<RefCell<liquid::Object>>,
    template_cfg: &mut config::Config,
    allow_commands: bool,
    silent: bool,
) -> Result<()> {
    let engine = create_rhai_engine(dir, liquid_object, allow_commands, silent);
    evaluate_scripts(dir, &template_cfg.get_pre_hooks(), engine)
}

pub fn execute_post_hooks(
    dir: &Path,
    liquid_object: Rc<RefCell<liquid::Object>>,
    template_cfg: &config::Config,
    allow_commands: bool,
    silent: bool,
) -> Result<()> {
    let engine = create_rhai_engine(dir, liquid_object, allow_commands, silent);
    evaluate_scripts(dir, &template_cfg.get_post_hooks(), engine)
}

fn evaluate_scripts(dir: &Path, scripts: &[String], engine: rhai::Engine) -> Result<()> {
    let cwd = env::current_dir()?;
    let _ = CleanupJob::new(move || {
        env::set_current_dir(cwd).ok();
    });
    env::set_current_dir(dir)?;

    for script in scripts {
        engine
            .eval_file::<()>(script.into())
            .map_err(|e| anyhow::anyhow!(e.to_string()))
            .context(format!(
                "{} {} {}",
                emoji::ERROR,
                style("Failed executing script:").bold().red(),
                style(script.to_owned()).yellow(),
            ))?;
    }

    Ok(())
}

fn create_rhai_engine(
    dir: &Path,
    liquid_object: Rc<RefCell<liquid::Object>>,
    allow_commands: bool,
    silent: bool,
) -> rhai::Engine {
    let mut engine = rhai::Engine::new();

    // register modules
    let module = variable_mod::create_module(liquid_object);
    engine.register_static_module("variable", module.into());

    let module = file_mod::create_module(dir);
    engine.register_static_module("file", module.into());

    let module = system_mod::create_module(allow_commands, silent);
    engine.register_static_module("system", module.into());

    // register functions for changing case
    engine.register_fn("to_kebab_case", |str: &str| str.to_kebab_case());
    engine.register_fn("to_lower_camel_case", |str: &str| str.to_lower_camel_case());
    engine.register_fn("to_pascal_case", |str: &str| str.to_pascal_case());
    engine.register_fn("to_shouty_kebab_case", |str: &str| {
        str.to_shouty_kebab_case()
    });
    engine.register_fn("to_shouty_snake_case", |str: &str| {
        str.to_shouty_snake_case()
    });
    engine.register_fn("to_snake_case", |str: &str| str.to_snake_case());
    engine.register_fn("to_title_case", |str: &str| str.to_title_case());
    engine.register_fn("to_upper_camel_case", |str: &str| str.to_upper_camel_case());

    // other free-standing functions
    engine.register_result_fn("abort", |error: &str| -> HookResult<String> {
        Err(error.into())
    });

    engine
}
