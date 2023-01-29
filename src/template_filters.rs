#![allow(clippy::box_default)]

use ::liquid_core::error::Error;
use anyhow::Result;
use console::style;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTitleCase, ToUpperCamelCase,
};
use liquid::model;
use liquid_core::{parser::FilterArguments, Filter, ParseFilter, Runtime, Value, ValueView};
use liquid_derive::FilterReflection;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    emoji,
    hooks::{create_rhai_engine, PoisonError},
    template::LiquidObjectResource,
};

macro_rules! create_case_filter {
    ($name:literal, $kebab_name:ident, $expr:expr) => {
        paste::paste! {
            #[derive(Clone, ParseFilter, FilterReflection)]
            #[filter(
                name = $name,
                description = "Change text to " $name,
                parsed([<$kebab_name Filter>])
            )]
            pub struct [<$kebab_name Filter Parser>];

            #[derive(Debug, Default, liquid_derive::Display_filter)]
            #[name = $name]
            struct [<$kebab_name Filter>];

            impl Filter for [<$kebab_name Filter>] {
                fn evaluate(
                    &self,
                    input: &dyn ValueView,
                    _runtime: &dyn Runtime,
                ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
                    let input = input
                        .as_scalar()
                        .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

                    let input = $expr(input.into_string().to_string());
                    Ok(liquid_core::model::Value::scalar(input))
                }
            }
        }
    };
}

create_case_filter!("kebab_case", KebabCase, |i: String| i.to_kebab_case());
create_case_filter!("lower_camel_case", LowerCamelCase, |i: String| i
    .to_lower_camel_case());
create_case_filter!("pascal_case", PascalCase, |i: String| i.to_pascal_case());
create_case_filter!("shouty_kebab_case", ShoutyKebabCase, |i: String| i
    .to_shouty_kebab_case());
create_case_filter!("shouty_snake_case", ShoutySnakeCase, |i: String| i
    .to_shouty_snake_case());
create_case_filter!("snake_case", SnakeCase, |i: String| i.to_snake_case());
create_case_filter!("title_case", TitleCase, |i: String| i.to_title_case());
create_case_filter!("upper_camel_case", UpperCamelCase, |i: String| i
    .to_upper_camel_case());

#[derive(Clone, FilterReflection)]
#[filter(
    name = "rhai",
    description = "Run Rhai script as a filter",
    parsed(RhaiFilter)
)]
pub struct RhaiFilterParser {
    template_dir: PathBuf,
    liquid_object: LiquidObjectResource,
    allow_commands: bool,
    silent: bool,
    rhai_filter_files: Arc<Mutex<Vec<PathBuf>>>,
}

impl RhaiFilterParser {
    pub fn new(
        template_dir: PathBuf,
        liquid_object: LiquidObjectResource,
        allow_commands: bool,
        silent: bool,
        rhai_filter_files: Arc<Mutex<Vec<PathBuf>>>,
    ) -> Self {
        Self {
            template_dir,
            liquid_object,
            allow_commands,
            silent,
            rhai_filter_files,
        }
    }
}

impl ParseFilter for RhaiFilterParser {
    fn parse(&self, mut args: FilterArguments) -> liquid_core::Result<Box<dyn Filter>> {
        if args.positional.next().is_some() {
            return Err(Error::with_msg("Invalid number of positional arguments")
                .context("cause", concat!("expected at most 0 positional arguments")));
        }
        if let Some(arg) = args.keyword.next() {
            return Err(Error::with_msg(format!(
                "Unexpected named argument `{}`",
                arg.0
            )));
        }
        Ok(Box::new(RhaiFilter {
            template_dir: self.template_dir.clone(),
            liquid_object: self.liquid_object.clone(),
            allow_commands: self.allow_commands,
            silent: self.silent,
            rhai_filter_files: self.rhai_filter_files.clone(),
        }))
    }

    fn reflection(&self) -> &dyn liquid_core::FilterReflection {
        self
    }
}

#[derive(Debug, liquid_derive::Display_filter)]
#[name = "rhai"]
struct RhaiFilter {
    template_dir: PathBuf,
    liquid_object: LiquidObjectResource,
    allow_commands: bool,
    silent: bool,
    rhai_filter_files: Arc<Mutex<Vec<PathBuf>>>,
}

impl Filter for RhaiFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<Value, liquid_core::Error> {
        // Unfortunately, liquid filters can't really cause liquid to fail. It just leaves the
        // substitution as is - thus we resort to displaying warnings to the user.

        let engine = create_rhai_engine(
            &self.template_dir,
            &self.liquid_object,
            self.allow_commands,
            self.silent,
        );
        let file_path = PathBuf::from(input.to_kstr().to_string());
        if !file_path.exists() {
            eprintln!(
                "{} {} {} {}",
                emoji::WARN,
                style("Filter script").bold().yellow(),
                style(file_path.display()).bold().red(),
                style("not found").bold().yellow(),
            );
            return Err(liquid_core::Error::with_msg(format!(
                "Filter script {} not found",
                file_path.display()
            )));
        }
        self.rhai_filter_files
            .lock()
            .map_err(|_| liquid_core::Error::with_msg(PoisonError.to_string()))?
            .push(file_path.clone());

        let result = engine.eval_file::<String>(file_path.clone());
        match result {
            Ok(r) => Ok(Value::Scalar(model::Scalar::from(r))),
            Err(err) => {
                eprintln!(
                    "{} {} {} {} {}",
                    emoji::WARN,
                    style("Filter script").bold().yellow(),
                    style(file_path.display()).bold().red(),
                    style("contained error").bold().yellow(),
                    style(err.to_string()).bold().red(),
                );
                Err(liquid_core::Error::with_msg(err.to_string()))
            }
        }
    }
}
