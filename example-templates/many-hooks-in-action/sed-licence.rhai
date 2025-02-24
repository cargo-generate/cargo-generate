let pwd = system::command("pwd");
print(`PWD: ${pwd}`);

let working_directory = env::working_directory;
print(`working_directory: ${working_directory}`);

let destination_directory = env::destination_directory;
print(`destination_directory: ${destination_directory}`);

// replace the year in the license file
system::command("gsed", ["-i", "s/2018/2022/g", "LICENSE"]);

let year = system::date().year;
// replace the year in the license file, with a variable interpolation
system::command("gsed", ["-i", `s/2022/${year}/g`, "LICENSE"]);

print("Listing files in the current directory, with 'file::listdir()'");
let files = file::listdir();
for f in files {
    print(`file: ${f}`);
}

// this is actually the same as above, the path must be inside the template directory, cannot be absolute or ourside
print("Listing files in the current directory, with 'file::listdir('.')'");
let files = file::listdir(".");
for f in files {
    print(`file: ${f}`);
}
