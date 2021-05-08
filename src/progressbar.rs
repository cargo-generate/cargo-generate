use indicatif::{MultiProgress, ProgressStyle};

pub(crate) fn new() -> MultiProgress {
    MultiProgress::new()
}

pub(crate) fn spinner() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
}
