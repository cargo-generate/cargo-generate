// This example script asks and reacts to the the users selection of license

let license = variable::get("license").to_lower();
while switch license {
    "none" => {
        debug("you selected to not use any license");
        file::delete("LICENSE-APACHE");
        file::delete("LICENSE-MIT");
        false
    }
    "mit" => {
        debug("you selected to use the MIT license");
        file::delete("LICENSE-APACHE");
        file::rename("LICENSE-MIT", "LICENSE");
        false
    }
    "apache" => {
        debug("you selected to use the APACHE license");
        file::rename("LICENSE-APACHE", "LICENSE");
        file::delete("LICENSE-MIT");
        false
    }
    _ => true,
} {
    license = variable::prompt("Which license do you want to use?", "none", ["none", "apache", "mit"]);
}
variable::set("license", license);
