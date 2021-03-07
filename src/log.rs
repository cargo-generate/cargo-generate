#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::emoji::WARN,
            format!($($arg)*)
        );
    })
}
