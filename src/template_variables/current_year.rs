use chrono::Datelike;

pub fn get_current_year() -> i32 {
    chrono::Local::now().date().year()
}
