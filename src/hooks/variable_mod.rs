use liquid::ValueView;
use liquid_core::Value;
use regex::Regex;
use rhai::{Array, Dynamic, Module};

use crate::interactive::prompt_and_check_variable;
use crate::project_variables::{StringEntry, StringKind, TemplateSlots, VarInfo};
use crate::template::LiquidObjectResource;

use super::{HookResult, PoisonError};

pub fn create_module(liquid_object: &LiquidObjectResource) -> Module {
    let mut module = Module::new();

    module.set_native_fn("is_set", {
        let liquid_object = liquid_object.clone();
        move |name: &str| -> HookResult<bool> {
            match liquid_object.get_value(name)? {
                NamedValue::NonExistent => Ok(false),
                _ => Ok(true),
            }
        }
    });

    module.set_native_fn("get", {
        let liquid_object = liquid_object.clone();
        move |name: &str| -> HookResult<Dynamic> {
            match liquid_object.get_value(name)? {
                NamedValue::NonExistent => Ok(Dynamic::from(String::from(""))),
                NamedValue::Bool(v) => Ok(Dynamic::from(v)),
                NamedValue::String(v) => Ok(Dynamic::from(v)),
                NamedValue::Array(arr) => {
                    let rhai_array: Array = arr
                        .into_iter()
                        .map(liquid_to_rhai_value)
                        .collect::<HookResult<_>>()?;
                    Ok(Dynamic::from(rhai_array))
                }
            }
        }
    });

    module.set_native_fn("set", {
        let liquid_object = liquid_object.clone();
        move |name: &str, value: &str| -> HookResult<()> {
            match liquid_object.get_value(name)? {
                NamedValue::NonExistent | NamedValue::String(_) => {
                    liquid_object
                        .lock()
                        .map_err(|_| PoisonError::new_eval_alt_result())?
                        .borrow_mut()
                        .insert(
                            name.to_string().into(),
                            Value::Scalar(value.to_string().into()),
                        );
                    Ok(())
                }
                _ => Err(format!("Variable {name} not a String").into()),
            }
        }
    });

    module.set_native_fn("set", {
        let liquid_object = liquid_object.clone();
        move |name: &str, value: bool| -> HookResult<()> {
            match liquid_object.get_value(name)? {
                NamedValue::NonExistent | NamedValue::Bool(_) => {
                    liquid_object
                        .lock()
                        .map_err(|_| PoisonError::new_eval_alt_result())?
                        .borrow_mut()
                        .insert(name.to_string().into(), Value::Scalar(value.into()));
                    Ok(())
                }
                _ => Err(format!("Variable {name} not a bool").into()),
            }
        }
    });

    module.set_native_fn("set", {
        let liquid_object = liquid_object.clone();
        move |name: &str, value: Array| -> HookResult<()> {
            match liquid_object.get_value(name)? {
                NamedValue::NonExistent | NamedValue::Array(_) => {
                    let val = rhai_to_liquid_value(Dynamic::from(value))?;
                    liquid_object
                        .lock()
                        .map_err(|_| PoisonError::new_eval_alt_result())?
                        .borrow_mut()
                        .insert(name.to_string().into(), val);
                    Ok(())
                }
                _ => Err(format!("Variable {name} not an array").into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: bool| -> HookResult<bool> {
            let value = prompt_and_check_variable(
                &TemplateSlots {
                    prompt: prompt.into(),
                    var_name: "".into(),
                    var_info: VarInfo::Bool {
                        default: Some(default_value),
                    },
                },
                None,
            );

            match value {
                Ok(v) => Ok(v.parse::<bool>().map_err(|_| "Unable to parse into bool")?),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str| -> HookResult<String> {
            let value = prompt_and_check_variable(
                &TemplateSlots {
                    prompt: prompt.into(),
                    var_name: "".into(),
                    var_info: VarInfo::String {
                        entry: Box::new(StringEntry {
                            default: None,
                            kind: StringKind::String,
                            regex: None,
                        }),
                    },
                },
                None,
            );

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: &str| -> HookResult<String> {
            let value = prompt_and_check_variable(
                &TemplateSlots {
                    prompt: prompt.into(),
                    var_name: "".into(),
                    var_info: VarInfo::String {
                        entry: Box::new(StringEntry {
                            default: Some(default_value.into()),
                            kind: StringKind::String,
                            regex: None,
                        }),
                    },
                },
                None,
            );

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: &str, regex: &str| -> HookResult<String> {
            let value = prompt_and_check_variable(
                &TemplateSlots {
                    prompt: prompt.into(),
                    var_name: "".into(),
                    var_info: VarInfo::String {
                        entry: Box::new(StringEntry {
                            default: Some(default_value.into()),
                            kind: StringKind::String,
                            regex: Some(Regex::new(regex).map_err(|_| "Invalid regex")?),
                        }),
                    },
                },
                None,
            );

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: &str, choices: rhai::Array| -> HookResult<String> {
            let value = prompt_and_check_variable(
                &TemplateSlots {
                    prompt: prompt.into(),
                    var_name: "".into(),
                    var_info: VarInfo::String {
                        entry: Box::new(StringEntry {
                            default: Some(default_value.into()),
                            kind: StringKind::Choices(
                                choices
                                    .iter()
                                    .map(|d| d.to_owned().into_string().unwrap())
                                    .collect(),
                            ),
                            regex: None,
                        }),
                    },
                },
                None,
            );

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module
}

enum NamedValue {
    NonExistent,
    Bool(bool),
    String(String),
    Array(Vec<Value>),
}

trait GetNamedValue {
    fn get_value(&self, name: &str) -> HookResult<NamedValue>;
}

impl GetNamedValue for LiquidObjectResource {
    fn get_value(&self, name: &str) -> HookResult<NamedValue> {
        let value = self
            .lock()
            .map_err(|_| PoisonError::new_eval_alt_result())?
            .borrow()
            .get(name)
            .map_or(NamedValue::NonExistent, |value| {
                // Check if it's an array first
                if let Some(arr) = value.as_array() {
                    let values: Vec<Value> = arr.values().map(|v| v.to_value()).collect();
                    return NamedValue::Array(values);
                }

                // Then check if it's a scalar
                value
                    .as_scalar()
                    .map(|scalar| {
                        scalar.to_bool().map_or_else(
                            || {
                                let v = scalar.to_kstr();
                                NamedValue::String(String::from(v.as_str()))
                            },
                            NamedValue::Bool,
                        )
                    })
                    .unwrap_or_else(|| NamedValue::NonExistent)
            });
        Ok(value)
    }
}

fn rhai_to_liquid_value(val: Dynamic) -> HookResult<Value> {
    val.as_bool()
        .map(Into::into)
        .map(Value::Scalar)
        .or_else(|_| val.clone().into_string().map(Into::into).map(Value::Scalar))
        .or_else(|_| {
            val.clone()
                .try_cast::<Array>()
                .ok_or_else(|| {
                    format!(
                        "expecting type to be string, bool or array but found a '{}' instead",
                        val.type_name()
                    )
                    .into()
                })
                .and_then(|arr| {
                    arr.into_iter()
                        .map(rhai_to_liquid_value)
                        .collect::<HookResult<_>>()
                        .map(Value::Array)
                })
        })
}

fn liquid_to_rhai_value(val: Value) -> HookResult<Dynamic> {
    match val {
        Value::Scalar(scalar) => {
            // Try to convert to bool first, then to string
            scalar.to_bool().map_or_else(
                || Ok(Dynamic::from(String::from(scalar.to_kstr().as_str()))),
                |b| Ok(Dynamic::from(b)),
            )
        }
        Value::Array(arr) => {
            let rhai_array: Array = arr
                .into_iter()
                .map(liquid_to_rhai_value)
                .collect::<HookResult<_>>()?;
            Ok(Dynamic::from(rhai_array))
        }
        _ => Err(format!(
            "unsupported liquid value type for conversion to rhai: {:?}",
            val
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
    };

    use liquid::Object;

    use super::*;

    #[test]
    fn test_rhai_set() {
        let mut engine = rhai::Engine::new();
        let liquid_object = Arc::new(Mutex::new(RefCell::new(Object::new())));

        let module = create_module(&liquid_object);
        engine.register_static_module("variable", module.into());

        engine
            .eval::<()>(
                r#"
            let dependencies = ["some_dep", "other_dep"];

            variable::set("dependencies", dependencies);
        "#,
            )
            .unwrap();

        let ref_cell = liquid_object.lock().unwrap();
        let liquid_object = ref_cell.borrow();

        assert_eq!(
            liquid_object.get("dependencies"),
            Some(&Value::Array(vec![
                Value::Scalar("some_dep".into()),
                Value::Scalar("other_dep".into())
            ]))
        );
    }

    #[test]
    fn test_rhai_get_array() {
        let mut engine = rhai::Engine::new();
        let mut obj = Object::new();

        // Pre-populate the liquid object with an array
        obj.insert(
            "test_array".into(),
            Value::Array(vec![
                Value::Scalar("aaa".into()),
                Value::Scalar("bbb".into()),
                Value::Scalar("ccc".into()),
                Value::Scalar("ddd".into()),
            ]),
        );

        let liquid_object = Arc::new(Mutex::new(RefCell::new(obj)));
        let module = create_module(&liquid_object);
        engine.register_static_module("variable", module.into());

        // Test is_set() on array variable
        let is_set: bool = engine
            .eval(
                r#"
            variable::is_set("test_array")
        "#,
            )
            .unwrap();
        assert!(is_set);

        // Test get() on array variable and iterate
        let result: String = engine
            .eval(
                r#"
            let arr = variable::get("test_array");
            let result = "";
            for item in arr {
                result += item + ",";
            }
            result
        "#,
            )
            .unwrap();
        assert_eq!(result, "aaa,bbb,ccc,ddd,");
    }

    #[test]
    fn test_rhai_get_nonexistent_array() {
        let mut engine = rhai::Engine::new();
        let liquid_object = Arc::new(Mutex::new(RefCell::new(Object::new())));

        let module = create_module(&liquid_object);
        engine.register_static_module("variable", module.into());

        // Test is_set() on non-existent variable
        let is_set: bool = engine
            .eval(
                r#"
            variable::is_set("nonexistent")
        "#,
            )
            .unwrap();
        assert!(!is_set);

        // Test get() on non-existent variable returns empty string
        let result: String = engine
            .eval(
                r#"
            variable::get("nonexistent")
        "#,
            )
            .unwrap();
        assert_eq!(result, "");
    }
}
