use rhai::{Dynamic, Module};
use std::{collections::BTreeMap, process::Command};
use time::OffsetDateTime;

use crate::{
    interactive::prompt_and_check_variable,
    project_variables::{StringEntry, StringKind, TemplateSlots, VarInfo},
};

use super::HookResult;

/// Creates the system module, containing the `command` function,
/// which allows you to run system command.
pub fn create_module(allow_commands: bool, silent: bool) -> Module {
    let mut module = Module::new();

    module.set_native_fn("command", move |name: &str, commands_args: rhai::Array| {
        run_command(name, commands_args, allow_commands, silent)
    });
    module.set_native_fn("date", get_utc_date);

    module
}

fn run_command(
    name: &str,
    args: rhai::Array,
    allow_commands: bool,
    silent: bool,
) -> HookResult<Dynamic> {
    let args: Vec<String> = args.into_iter().map(|arg| arg.to_string()).collect();

    // If --allow-commands is false, we need to prompt. But we shouldn't if we're in silent mode.
    if !allow_commands && silent {
        return Err("Cannot prompt for system command confirmation in silent mode. Use --allow-commands if you want to allow the template to run system commands in silent mode.".into());
    }

    let full_command = if args.is_empty() {
        name.into()
    } else {
        format!("{name} {}", args.join(" "))
    };

    // If the user specified the --allow-commands flag, don't prompt.
    let should_run = allow_commands
        || {
            let prompt = format!("The template is requesting to run the following command. Do you agree?\n{full_command}");

            // Prompt the user for whether they actually want to run the command.
            let value = prompt_and_check_variable(
                &TemplateSlots {
                    prompt: prompt.into(),
                    var_name: "".into(),
                    var_info: VarInfo::String {
                        entry: Box::new(StringEntry {
                            default: Some("no".into()),
                            kind: StringKind::Choices(vec!["yes".into(), "no".into()]),
                            regex: None,
                        }),
                    },
                },
                None,
            );

            // Only accept clearly positive affirmations.
            matches!(
                value.map(|s| s.trim().to_ascii_lowercase()).as_deref(),
                Ok("yes" | "y")
            )
        };

    if !should_run {
        return Err(format!("User denied execution of system command `{full_command}`.").into());
    }

    let status = Command::new(name).args(args).status();

    match status {
        Ok(status) => {
            if status.success() {
                Ok(Dynamic::UNIT)
            } else {
                Err(
                    format!("System command `{full_command}` returned non-zero status: {status}")
                        .into(),
                )
            }
        }
        Err(e) => Err(format!("System command `{full_command}` failed to execute: {e}").into()),
    }
}

fn get_utc_date() -> HookResult<Dynamic> {
    Ok(construct_date_map(OffsetDateTime::now_utc()))
}

fn construct_date_map(dt: OffsetDateTime) -> Dynamic {
    let mut value = BTreeMap::new();
    value.insert(
        "year".parse().unwrap(),
        Dynamic::from_int(i64::from(dt.year())),
    );
    value.insert(
        "month".parse().unwrap(),
        Dynamic::from_int(dt.month() as i64),
    );
    value.insert(
        "day".parse().unwrap(),
        Dynamic::from_int(i64::from(dt.day())),
    );
    Dynamic::from_map(value)
}
