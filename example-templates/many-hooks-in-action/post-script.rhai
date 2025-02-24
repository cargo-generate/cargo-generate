print("You have reached a Rhai script, running as POST by cargo-generate");

let pwd = system::command("pwd");
print(`PWD: ${pwd}`);

let tmp = env::tmp;
print(`TMP Dir: ${tmp}`);

// this command fails
// system::command("foobarbak", ["this one fails.."]);
