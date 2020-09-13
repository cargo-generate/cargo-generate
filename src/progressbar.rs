use indicatif::ProgressBar;

pub(crate) fn new() -> ProgressBar {
    ProgressBar::new_spinner()
}
