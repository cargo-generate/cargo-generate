//
let pwd = system::command("pwd");
print("PWD: ${pwd}");

let tmp = env::tmp;
print("TMP: ${tmp}");

// replace the year in the license file
system::command("gsed", ["-i", "s/2018/2022/g", "LICENSE"]);

let year = system::date().year;
// replace the year in the license file, with a variable interpolation
system::command("gsed", ["-i", `s/2022/${year}/g`, "LICENSE"]);

let files = file::listdir(".");
for f in files {
    print(`file: ${f}`);
}
