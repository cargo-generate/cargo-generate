use anyhow::Result;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToTitleCase, ToUpperCamelCase,
};
use liquid_core::{Filter, ParseFilter, Runtime, ValueView};
use liquid_derive::FilterReflection;

macro_rules! create_fiter_case {
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

create_fiter_case!("kebab_case", KebabCase, |i: String| i.to_kebab_case());
create_fiter_case!("lower_camel_case", LowerCamelCase, |i: String| i
    .to_lower_camel_case());
create_fiter_case!("pascal_case", PascalCase, |i: String| i.to_pascal_case());
create_fiter_case!("shouty_kebab_case", ShoutyKebabCase, |i: String| i
    .to_shouty_kebab_case());
create_fiter_case!("shouty_snake_case", ShoutySnakeCase, |i: String| i
    .to_shouty_snake_case());
create_fiter_case!("snake_case", SnakeCase, |i: String| i.to_snake_case());
create_fiter_case!("title_case", TitleCase, |i: String| i.to_title_case());
create_fiter_case!("upper_camel_case", UpperCamelCase, |i: String| i
    .to_upper_camel_case());
