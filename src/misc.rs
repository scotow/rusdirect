#[macro_export]
macro_rules! exit_error {
    ($($arg:tt)+) => {
        {
            log::error!($($arg)+);
            std::process::exit(1)
        }
    }
}
