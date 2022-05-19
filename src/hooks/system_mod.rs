use rhai::{Dynamic, EvalAltResult, Module};
use std::process::Command;

use crate::{
    interactive::prompt_for_variable,
    project_variables::{StringEntry, TemplateSlots, VarInfo},
    Args,
};

type Result<T> = anyhow::Result<T, Box<EvalAltResult>>;

/// Creates the system module, containing the `command` function,
/// which allows you to run system command.
pub fn create_module(args: &Args) -> Module {
    let mut module = Module::new();

    let allow_commands = args.allow_commands;
    module.set_native_fn("command", move |name: &str, commands_args: rhai::Array| {
        run_command(name, commands_args, allow_commands)
    });

    module
}

fn run_command(name: &str, args: rhai::Array, allow_commands: bool) -> Result<Dynamic> {
    let args: Vec<String> = args.into_iter().map(|arg| arg.to_string()).collect();

    // If the user specified the --allow-commands flag, don't prompt.
    let should_run = allow_commands || {
        let prompt = format!(
            "The template is requesting to run the following command. Do you agree?\n{} {}\n",
            name,
            args.join(" ")
        );

        // Prompt the user for whether they actually want to run the command.
        let value = prompt_for_variable(&TemplateSlots {
            prompt,
            var_name: "".into(),
            var_info: VarInfo::String {
                entry: Box::new(StringEntry {
                    default: Some("no".into()),
                    choices: Some(vec!["yes".into(), "no".into()]),
                    regex: None,
                }),
            },
        });

        // Only accept clearly positive affirmations.
        matches!(
            value.map(|s| s.trim().to_ascii_lowercase()).as_deref(),
            Ok("yes" | "y")
        )
    };

    if !should_run {
        return Ok(Dynamic::UNIT);
    }

    let output = Command::new(name).args(args).output();

    Ok(match output {
        Ok(o) => Dynamic::from_blob(o.stdout),
        Err(_) => Dynamic::UNIT,
    })
}
