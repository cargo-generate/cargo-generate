use liquid::{Object, ValueView};
use liquid_core::Value;
use regex::Regex;
use rhai::{Array, Dynamic, Module};
use std::cell::RefCell;
use std::rc::Rc;

use crate::interactive::prompt_and_check_variable;
use crate::project_variables::{StringEntry, TemplateSlots, VarInfo};

use super::HookResult;

pub fn create_module(liquid_object: Rc<RefCell<Object>>) -> Module {
    let mut module = Module::new();

    module.set_native_fn("is_set", {
        let liquid_object = liquid_object.clone();
        move |name: &str| -> HookResult<bool> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant => Ok(false),
                _ => Ok(true),
            }
        }
    });

    module.set_native_fn("get", {
        let liquid_object = liquid_object.clone();
        move |name: &str| -> HookResult<Dynamic> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant => Ok(Dynamic::from(String::from(""))),
                NamedValue::Bool(v) => Ok(Dynamic::from(v)),
                NamedValue::String(v) => Ok(Dynamic::from(v)),
            }
        }
    });

    module.set_native_fn("set", {
        let liquid_object = liquid_object.clone();

        move |name: &str, value: &str| -> HookResult<()> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant | NamedValue::String(_) => {
                    liquid_object.borrow_mut().insert(
                        name.to_string().into(),
                        Value::Scalar(value.to_string().into()),
                    );
                    Ok(())
                }
                _ => Err(format!("Variable {} not a String", name).into()),
            }
        }
    });

    module.set_native_fn("set", {
        let liquid_object = liquid_object.clone();

        move |name: &str, value: bool| -> HookResult<()> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant | NamedValue::Bool(_) => {
                    liquid_object
                        .borrow_mut()
                        .insert(name.to_string().into(), Value::Scalar(value.into()));
                    Ok(())
                }
                _ => Err(format!("Variable {} not a bool", name).into()),
            }
        }
    });

    module.set_native_fn("set", {
        move |name: &str, value: Array| -> HookResult<()> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant => {
                    let val = rhai_to_liquid_value(Dynamic::from(value))?;
                    liquid_object
                        .borrow_mut()
                        .insert(name.to_string().into(), val);
                    Ok(())
                }
                _ => Err(format!("Variable {} not an array", name).into()),
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
                            choices: None,
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
                            choices: None,
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
                            choices: None,
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
                            choices: Some(
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
    NonExistant,
    Bool(bool),
    String(String),
}

trait GetNamedValue {
    fn get_value(&self, name: &str) -> NamedValue;
}

impl GetNamedValue for Rc<RefCell<Object>> {
    fn get_value(&self, name: &str) -> NamedValue {
        self.borrow()
            .get(name)
            .map_or(NamedValue::NonExistant, |value| {
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
                    .unwrap_or_else(|| NamedValue::NonExistant)
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhai_set() {
        let mut engine = rhai::Engine::new();
        let liquid_object = Rc::new(RefCell::new(liquid::Object::new()));

        let module = create_module(liquid_object.clone());
        engine.register_static_module("variable", module.into());

        engine
            .eval::<()>(
                r#"
            let dependencies = ["some_dep", "other_dep"];

            variable::set("dependencies", dependencies);
        "#,
            )
            .unwrap();

        let liquid_object = liquid_object.borrow();

        assert_eq!(
            liquid_object.get("dependencies"),
            Some(&Value::Array(vec![
                Value::Scalar("some_dep".into()),
                Value::Scalar("other_dep".into())
            ]))
        );
    }
}
