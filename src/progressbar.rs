use indicatif::ProgressBar;

pub fn new() -> ProgressBar {
    ProgressBar::new_spinner()
}
