use anyhow::Result;
use heck::{CamelCase, KebabCase, SnakeCase};
use liquid_core::{Filter, ParseFilter, Runtime, ValueView};
use liquid_derive::FilterReflection;

pub static CONDITIONAL_FALSE_VALUES: [&str; 4] = ["0", "false", "none", "no"];

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "is_bin",
    description = "returns 'true' if variable is 'bin'",
    parsed(IsBinFilter)
)]
pub struct IsBinFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "is_bin"]
struct IsBinFilter;

impl Filter for IsBinFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;
        let input = input.into_string().to_string().to_lowercase();
        let input = input.trim();

        Ok(liquid_core::model::Value::scalar(if input == "bin" {
            "true"
        } else {
            "false"
        }))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "is_lib",
    description = "returns 'true' if variable is 'lib'|'dylib'|'staticlib'|'cdylib'|'rlib'",
    parsed(IsLibFilter)
)]
pub struct IsLibFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "is_lib"]
struct IsLibFilter;

impl Filter for IsLibFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;
        let input = input.into_string().to_string().to_lowercase();
        let input = input.trim();

        Ok(liquid_core::model::Value::scalar(
            if input == "lib"
                || input == "dylib"
                || input == "staticlib"
                || input == "cdylib"
                || input == "rlib"
            {
                "true"
            } else {
                "false"
            },
        ))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "is_macro",
    description = "returns 'true' if variable is 'proc-macro'",
    parsed(IsMacroFilter)
)]
pub struct IsMacroFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "is_macro"]
struct IsMacroFilter;

impl Filter for IsMacroFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;
        let input = input.into_string().to_string().to_lowercase();
        let input = input.trim();

        Ok(liquid_core::model::Value::scalar(
            if input == "proc-macro" {
                "true"
            } else {
                "false"
            },
        ))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "truthy",
    description = "returns 'true' if variable is considered truthy",
    parsed(TruthyFilter)
)]
pub struct TruthyFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "truthy"]
struct TruthyFilter;

impl Filter for TruthyFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;
        let input = input.into_string().to_string().to_lowercase();
        let input = input.trim();

        Ok(liquid_core::model::Value::scalar(
            if !input.is_empty() && !CONDITIONAL_FALSE_VALUES.contains(&input) {
                "true"
            } else {
                ""
            },
        ))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "falsy",
    description = "returns 'true' if variable is considered falsy",
    parsed(FalsyFilter)
)]
pub struct FalsyFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "falsy"]
struct FalsyFilter;

impl Filter for FalsyFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;
        let input = input.into_string().to_string().to_kebab_case();
        let input = input.trim();

        Ok(liquid_core::model::Value::scalar(
            if input.is_empty() || CONDITIONAL_FALSE_VALUES.contains(&input) {
                "true"
            } else {
                ""
            },
        ))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "kebab_case",
    description = "Change text to kebab-case.",
    parsed(KebabCaseFilter)
)]
pub struct KebabCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "kebab_case"]
struct KebabCaseFilter;

impl Filter for KebabCaseFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid_core::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_string().to_kebab_case();
        Ok(liquid_core::model::Value::scalar(input))
    }
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "pascal_case",
    description = "Change text to PascalCase.",
    parsed(PascalCaseFilter)
)]
pub struct PascalCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "pascal_case"]
struct PascalCaseFilter;

impl Filter for PascalCaseFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_camel_case();
        Ok(liquid::model::Value::scalar(input))
    }
}

#[derive(Clone, liquid_derive::ParseFilter, liquid_derive::FilterReflection)]
#[filter(
    name = "snake_case",
    description = "Change text to snake_case.",
    parsed(SnakeCaseFilter)
)]
pub struct SnakeCaseFilterParser;

#[derive(Debug, Default, liquid_derive::Display_filter)]
#[name = "snake_case"]
struct SnakeCaseFilter;

impl Filter for SnakeCaseFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> Result<liquid::model::Value, liquid_core::error::Error> {
        let input = input
            .as_scalar()
            .ok_or_else(|| liquid_core::error::Error::with_msg("String expected"))?;

        let input = input.into_string().to_snake_case();
        Ok(input.to_value())
    }
}
