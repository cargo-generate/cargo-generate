use anyhow::{Context, Result};
use console::style;
use rhai::EvalAltResult;
use std::cell::RefCell;
use std::rc::Rc;
use std::{env, path::Path};

use crate::config;
use crate::emoji;

mod file_mod;
mod variable_mod;

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
    liquid_object: &liquid::Object,
    template_cfg: &mut config::Config,
) -> Result<()> {
    let engine = create_rhai_engine(dir, liquid_object);
    evaluate_scripts(dir, &template_cfg.get_pre_hooks(), engine)
}

pub fn execute_post_hooks(
    dir: &Path,
    liquid_object: &liquid::Object,
    template_cfg: &config::Config,
) -> Result<()> {
    let engine = create_rhai_engine(dir, liquid_object);
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

fn create_rhai_engine(dir: &Path, liquid_object: &liquid::Object) -> rhai::Engine {
    let liquid_object = Rc::new(RefCell::new(liquid_object.clone()));
    let mut engine = rhai::Engine::new();

    let module = variable_mod::create_module(liquid_object);
    engine.register_static_module("variable", module.into());

    let module = file_mod::create_module(dir);
    engine.register_static_module("file", module.into());

    engine.register_result_fn(
        "abort",
        |error: &str| -> Result<String, Box<EvalAltResult>> { Err(error.into()) },
    );

    engine
}
