use anyhow::{Context, Result};
use console::style;
use env_mod::Environment;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTitleCase, ToUpperCamelCase,
};
use liquid::ValueView;
use log::debug;
use rhai::module_resolvers::FileModuleResolver;
use rhai::EvalAltResult;
use std::path::Path;
use std::path::PathBuf;

use crate::emoji;
use crate::template::LiquidObjectResource;

mod context;
mod env_mod;
mod file_mod;
mod system_mod;
mod variable_mod;

type HookResult<T> = std::result::Result<T, Box<EvalAltResult>>;

pub use context::RhaiHooksContext;

pub fn execute_hooks(context: &RhaiHooksContext, scripts: &[String]) -> Result<()> {
    debug!("executing rhai with context: {context:?}");

    let engine = create_rhai_engine(context);
    evaluate_scripts(&context.working_directory, scripts, engine)?;
    Ok(())
}

fn evaluate_scripts(template_dir: &Path, scripts: &[String], engine: rhai::Engine) -> Result<()> {
    for script in scripts {
        let script: PathBuf = template_dir.join(script);

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

    let var_mod = variable_mod::create_module(&context.liquid_object);
    let file_mod = file_mod::create_module(&context.working_directory);
    let system_mod = system_mod::create_module(
        context.working_directory.clone(),
        context.allow_commands,
        context.silent,
    );
    let env_mod = env_mod::create_module(Environment {
        working_directory: context.working_directory.clone(),
        destination_directory: context.destination_directory.clone(),
    });

    engine
        .set_module_resolver(FileModuleResolver::new_with_path(
            &context.working_directory,
        ))
        .register_static_module("variable", var_mod.into())
        .register_static_module("file", file_mod.into())
        .register_static_module("system", system_mod.into())
        .register_static_module("env", env_mod.into())
        // register functions for changing case
        .register_fn("to_kebab_case", |str: &str| str.to_kebab_case())
        .register_fn("to_lower_camel_case", |str: &str| str.to_lower_camel_case())
        .register_fn("to_pascal_case", |str: &str| str.to_pascal_case())
        .register_fn("to_shouty_kebab_case", |str: &str| {
            str.to_shouty_kebab_case()
        })
        .register_fn("to_shouty_snake_case", |str: &str| {
            str.to_shouty_snake_case()
        })
        .register_fn("to_snake_case", |str: &str| str.to_snake_case())
        .register_fn("to_title_case", |str: &str| str.to_title_case())
        .register_fn("to_upper_camel_case", |str: &str| str.to_upper_camel_case())
        // other free-standing functions
        .register_fn("abort", |error: &str| -> HookResult<String> {
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
