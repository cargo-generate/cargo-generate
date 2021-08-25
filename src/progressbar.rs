use indicatif::{MultiProgress, ProgressStyle};

pub fn new() -> MultiProgress {
    MultiProgress::new()
}

pub fn spinner() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
}
