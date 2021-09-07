// This example script asks and reacts to the the users selection of license

let license = variable::get("license").to_lower();
let license_ok = false;
do {
    license_ok = switch license {
        "none" => {
            debug("you selected to not use any license");
            file::delete("LICENSE-APACHE");
            file::delete("LICENSE-MIT");
            true
        }
        "mit" => {
            debug("you selected to use the MIT license");
            file::delete("LICENSE-APACHE");
            file::rename("LICENSE-MIT", "LICENSE");
            true
        }
        "apache" => {
            debug("you selected to use the APACHE license");
            file::rename("LICENSE-APACHE", "LICENSE");
            file::delete("LICENSE-MIT");
            true
        }
        _ => {
            license = variable::prompt("Which license do you want to use?", "none", ["none", "apache", "mit"]);
            false
        }
    }
} until license_ok;
variable::set("license", license);
