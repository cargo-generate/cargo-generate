use log::debug;
use rhai::{Dynamic, FuncRegistration, Module};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};
use time::OffsetDateTime;

use crate::{
    interactive::prompt_and_check_variable,
    project_variables::{StringEntry, StringKind, TemplateSlots, VarInfo},
};

use super::HookResult;

/// Creates the system module, containing the `command` function,
/// which allows you to run system command.
pub fn create_module(working_directory: PathBuf, allow_commands: bool, silent: bool) -> Module {
    let mut module = Module::new();

    let cwd = working_directory.clone();
    FuncRegistration::new("command").set_into_module(
        &mut module,
        move |name: &str, commands_args: rhai::Array| {
            run_command(&cwd, name, commands_args, allow_commands, silent)
        },
    );

    let cwd = working_directory.clone();
    FuncRegistration::new("command").set_into_module(&mut module, move |name: &str| {
        run_command(&cwd, name, rhai::Array::new(), allow_commands, silent)
    });

    module.set_native_fn("date", get_utc_date);

    module
}

fn run_command(
    working_directory: &Path,
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

    debug!(
        "the command is executed within the working directory: {}",
        working_directory.display()
    );

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(&full_command)
            .current_dir(working_directory)
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .current_dir(working_directory)
            .output()
    };

    debug!("Command Call: {output:?}");
    match output {
        Ok(output) if output.status.success() && output.status.code().unwrap_or(1) == 0 => {
            match output.stdout.len() {
                0 => Ok(Dynamic::UNIT),
                _ => {
                    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    debug!("Rhai output: {output_str}");

                    Ok(Dynamic::from(output_str))
                }
            }
        }
        Ok(output) => Err(format!(
            "System command `{full_command}` failed to execute: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into()),
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        hooks::{create_rhai_engine, RhaiHooksContext},
        template::LiquidObjectResource,
    };
    use rhai::Engine;
    use tempfile::TempDir;

    #[test]
    fn test_system_module() {
        let tmp_dir = TempDir::new().unwrap();
        let mut file1 = std::fs::File::create(tmp_dir.path().join("file1")).unwrap();
        file1.write_all(b"test1").unwrap();
        let context = RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: true,
            silent: true,
        };
        std::env::set_current_dir(&context.working_directory).unwrap();
        let engine = create_rhai_engine(&context);

        let pwd = engine.eval::<String>(r#"system::command("pwd")"#).unwrap();
        assert_eq!(
            pwd,
            pwd.trim_end(),
            "there should be no trailing whitespace or newline"
        );
        // for reasons on the windows GH runner we get this issue:
        // assertion `left == right` failed: the system::command should run in the context of the working_directory: C:\Users\RUNNER~1\AppData\Local\Temp\.tmpDXzQr1
        //   left: "/tmp/.tmpDXzQr1"
        //   right: "\\\\?\\C:\\Users\\runneradmin\\AppData\\Local\\Temp\\.tmpDXzQr1"
        #[cfg(target_family = "unix")]
        assert_eq!(
            pwd,
            tmp_dir.path().canonicalize().unwrap().to_str().unwrap(),
            "the system::command should run in the context of the working_directory: {}",
            tmp_dir.path().display()
        );

        #[cfg(target_family = "unix")]
        let content = engine
            .eval::<String>(r#"system::command("cat", ["file1"])"#)
            .unwrap();

        #[cfg(target_family = "windows")]
        let content = engine
            .eval::<String>(r#"system::command("type", ["file1"])"#)
            .unwrap();

        assert_eq!(content, "test1");
    }

    // Note, macos and linux have different error message behaviour for this case
    // macos: System command `nonexistent_command` failed to execute: No such file or directory (os error 2)
    // linux: System command `nonexistent_command` failed to execute: "
    //
    // root cause is that on linux stderr is empty, on macos it contains the error message
    #[test]
    #[should_panic(expected = "System command `nonexistent_command` failed to execute: ")]
    fn test_run_command_failure() {
        let tmp_dir = TempDir::new().unwrap();
        let context = RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: true,
            silent: true,
        };
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        engine
            .eval::<String>(r#"system::command("nonexistent_command")"#)
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot prompt for system command confirmation in silent mode. Use --allow-commands if you want to allow the template to run system commands in silent mode."
    )]
    fn test_run_command_silent_mode_denied() {
        let tmp_dir = TempDir::new().unwrap();
        let context = RhaiHooksContext {
            working_directory: tmp_dir.path().to_path_buf(),
            destination_directory: tmp_dir.path().join("destination").to_path_buf(),
            liquid_object: LiquidObjectResource::default(),
            allow_commands: false,
            silent: true,
        };
        let engine = create_rhai_engine(&context);
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        engine
            .eval::<String>(r#"system::command("echo", ["hello"])"#)
            .unwrap();
    }

    #[test]
    fn test_get_utc_date() {
        let tmp_dir = TempDir::new().unwrap();
        let mut engine = Engine::new();
        let module = super::create_module(tmp_dir.path().to_path_buf(), true, true);
        engine.register_static_module("system", module.into());

        let result = engine.eval::<rhai::Map>(r#"system::date()"#).unwrap();
        let now = time::OffsetDateTime::now_utc();
        assert!(result.contains_key("year"));
        assert!(result.contains_key("month"));
        assert!(result.contains_key("day"));

        assert_eq!(result["year"].as_int().unwrap(), i64::from(now.year()));
        assert_eq!(result["month"].as_int().unwrap(), now.month() as i64);
        assert_eq!(result["day"].as_int().unwrap(), i64::from(now.day()));
    }
}
