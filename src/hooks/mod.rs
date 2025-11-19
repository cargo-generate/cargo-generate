use anyhow::{Context, Result};
use console::style;
use env_mod::Environment;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTitleCase, ToUpperCamelCase,
};
use liquid::ValueView;
use log::debug;
use rhai::EvalAltResult;
use std::path::PathBuf;
use std::{env, path::Path};

use crate::emoji;
use crate::template::LiquidObjectResource;

mod context;
mod env_mod;
mod file_mod;
mod system_mod;
mod variable_mod;

type HookResult<T> = std::result::Result<T, Box<EvalAltResult>>;

struct CleanupJob<F: FnOnce()>(Option<F>);

pub use context::RhaiHooksContext;

impl<F: FnOnce()> CleanupJob<F> {
    pub const fn new(f: F) -> Self {
        Self(Some(f))
    }
}

impl<F: FnOnce()> Drop for CleanupJob<F> {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}

pub fn execute_hooks(context: &RhaiHooksContext, scripts: &[String]) -> Result<()> {
    debug!("executing rhai with context: {context:?}");

    let engine = create_rhai_engine(context);
    evaluate_scripts(&context.working_directory, scripts, engine)?;
    Ok(())
}

fn evaluate_scripts(template_dir: &Path, scripts: &[String], engine: rhai::Engine) -> Result<()> {
    let cwd = env::current_dir()?;
    let _ = CleanupJob::new(move || {
        env::set_current_dir(cwd).ok();
    });
    env::set_current_dir(template_dir)?;

    for script in scripts {
        let script: PathBuf = script.into();

        let result = engine
            .eval_file::<rhai::plugin::Dynamic>(script.clone())
            .map_err(|e| anyhow::anyhow!(e.to_string()))
            .with_context(|| {
                format!(
                    "{} {} {}",
                    emoji::ERROR,
                    style("Failed executing script:").bold().red(),
                    style(script.display()).yellow(),
                )
            })?;
        match result.into_string() {
            Ok(output) => {
                if !output.is_empty() {
                    println!(
                        "{} {} {}",
                        emoji::SPARKLE,
                        style(format!(
                            "Script `{}` executed successfully and returned output:",
                            script.display()
                        ))
                        .bold()
                        .green(),
                        style(output).yellow()
                    );
                } else {
                    println!(
                        "{} {}",
                        emoji::WRENCH,
                        style(format!(
                            "Script `{}` executed successfully with no output.",
                            script.display()
                        ))
                        .bold()
                        .green()
                    );
                }
            }
            Err(e) => {
                // this is not an issue, a rhai script can return nothing
                debug!(
                    "Script `{}` executed successfully but did not return a string: {e}",
                    script.display()
                );
            }
        }
    }

    Ok(())
}

pub fn evaluate_script<T: Clone + 'static>(
    liquid_object: &LiquidObjectResource,
    script: &str,
) -> HookResult<T> {
    let mut conditional_evaluation_engine = rhai::Engine::new();

    #[allow(deprecated)]
    conditional_evaluation_engine.on_var({
        let liquid_object = liquid_object.clone();
        move |name, _, _| {
            liquid_object
                .lock()
                .map_err(|_| PoisonError::new_eval_alt_result())?
                .borrow()
                .get(name)
                .map_or(Ok(None), |value| {
                    Ok(value.as_view().as_scalar().map(|scalar| {
                        scalar.to_bool().map_or_else(
                            || {
                                let v = scalar.to_kstr();
                                v.as_str().into()
                            },
                            |v| v.into(),
                        )
                    }))
                })
        }
    });

    conditional_evaluation_engine.eval_expression::<T>(script)
}

pub fn create_rhai_engine(context: &RhaiHooksContext) -> rhai::Engine {
    let mut engine = rhai::Engine::new();

    // register modules
    let module = variable_mod::create_module(&context.liquid_object);
    engine.register_static_module("variable", module.into());

    let module = file_mod::create_module(&context.working_directory);
    engine.register_static_module("file", module.into());

    let module = system_mod::create_module(
        context.working_directory.clone(),
        context.allow_commands,
        context.silent,
    );
    engine.register_static_module("system", module.into());

    let module = env_mod::create_module(Environment {
        working_directory: context.working_directory.clone(),
        destination_directory: context.destination_directory.clone(),
    });
    engine.register_static_module("env", module.into());

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
    engine.register_fn("abort", |error: &str| -> HookResult<String> {
        Err(error.into())
    });

    engine
}

#[derive(thiserror::Error, Debug)]
#[error("A lock was poisoned")]
pub struct PoisonError;

impl<E> From<std::sync::PoisonError<E>> for PoisonError {
    fn from(_: std::sync::PoisonError<E>) -> Self {
        Self
    }
}

impl PoisonError {
    pub fn new_eval_alt_result() -> EvalAltResult {
        EvalAltResult::ErrorSystem("".into(), Box::new(Self))
    }
}
