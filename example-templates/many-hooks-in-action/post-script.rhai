print("You have reached a Rhai script, running as POST by cargo-generate");

let pwd = system::command("pwd");
print(`[post-hook] pwd: ${pwd}`);

let working_directory = env::working_directory;
print(`[post-hook] working_directory: ${working_directory}`);

// this command will..
// system::command("foobarbak", ["this one fails.."]);
