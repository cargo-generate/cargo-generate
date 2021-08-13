#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::emoji::WARN,
            format!($($arg)*)
        );
    })
}
