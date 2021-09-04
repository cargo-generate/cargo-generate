use liquid::{Object, ValueView};
use liquid_core::Value;
use regex::Regex;
use rhai::{Dynamic, EvalAltResult, Module};
use std::cell::RefCell;
use std::rc::Rc;

use crate::interactive::prompt_for_variable;
use crate::project_variables::{StringEntry, TemplateSlots, VarInfo};

type Result<T> = anyhow::Result<T, Box<EvalAltResult>>;

pub fn create_module(liquid_object: Rc<RefCell<Object>>) -> Module {
    let mut module = Module::new();

    module.set_native_fn("is_set", {
        let liquid_object = liquid_object.clone();
        move |name: &str| -> Result<bool> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant => Ok(false),
                _ => Ok(true),
            }
        }
    });

    module.set_native_fn("get", {
        let liquid_object = liquid_object.clone();
        move |name: &str| -> Result<Dynamic> {
            match liquid_object.get_value(name) {
                NamedValue::NonExistant => Ok(Dynamic::from(String::from(""))),
                NamedValue::Bool(v) => Ok(Dynamic::from(v)),
                NamedValue::String(v) => Ok(Dynamic::from(v)),
            }
        }
    });

    module.set_native_fn("set", {
        let liquid_object = liquid_object.clone();

        move |name: &str, value: &str| -> Result<()> {
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
        move |name: &str, value: bool| -> Result<()> {
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

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: bool| -> Result<bool> {
            let value = prompt_for_variable(&TemplateSlots {
                prompt: prompt.into(),
                var_name: "".into(),
                var_info: VarInfo::Bool {
                    default: Some(default_value),
                },
            });

            match value {
                Ok(v) => Ok(v.parse::<bool>().map_err(|_| "Unable to parse into bool")?),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str| -> Result<String> {
            let value = prompt_for_variable(&TemplateSlots {
                prompt: prompt.into(),
                var_name: "".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: None,
                        choices: None,
                        regex: None,
                    }),
                },
            });

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: &str| -> Result<String> {
            let value = prompt_for_variable(&TemplateSlots {
                prompt: prompt.into(),
                var_name: "".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some(default_value.into()),
                        choices: None,
                        regex: None,
                    }),
                },
            });

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: &str, regex: &str| -> Result<String> {
            let value = prompt_for_variable(&TemplateSlots {
                prompt: prompt.into(),
                var_name: "".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some(default_value.into()),
                        choices: None,
                        regex: Some(Regex::new(regex).map_err(|_| "Invalid regex")?),
                    }),
                },
            });

            match value {
                Ok(v) => Ok(v),
                Err(e) => Err(e.to_string().into()),
            }
        }
    });

    module.set_native_fn("prompt", {
        move |prompt: &str, default_value: &str, choices: rhai::Array| -> Result<String> {
            let value = prompt_for_variable(&TemplateSlots {
                prompt: prompt.into(),
                var_name: "".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some(default_value.into()),
                        choices: Some(
                            choices
                                .iter()
                                .map(|d| d.to_owned().as_string().unwrap())
                                .collect(),
                        ),
                        regex: None,
                    }),
                },
            });

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
        match self.borrow().get(name) {
            Some(value) => value
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
                .unwrap_or_else(|| NamedValue::NonExistant),
            None => NamedValue::NonExistant,
        }
    }
}
