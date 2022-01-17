#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::emoji::WARN,
            format!($($arg)*)
        );
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::emoji::INFO,
            format!($($arg)*)
        );
    })
}
