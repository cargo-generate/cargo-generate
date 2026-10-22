use std::process::Command;

/// Panic with the cause when there is no working `rustfmt`.
///
/// The formatting suite asserts on what `rustfmt` did to a file. Without one,
/// the positive tests fail with a diff that names no cause, and the negative
/// ones pass for the wrong reason - nothing was formatted because nothing
/// could be. A missing rustup component leaves the `rustfmt` shim on `PATH`,
/// so a non-zero exit counts as missing too, not just a missing binary.
pub fn require_rustfmt() {
    let reason = match Command::new("rustfmt").arg("--version").output() {
        Ok(output) if output.status.success() => return,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            if stderr.is_empty() {
                format!("`rustfmt --version` exited with {}", output.status)
            } else {
                stderr
            }
        }
        Err(e) => e.to_string(),
    };

    panic!(
        "this test needs a working rustfmt and there is none: {reason}\n\
         install it with `rustup component add rustfmt`"
    );
}
