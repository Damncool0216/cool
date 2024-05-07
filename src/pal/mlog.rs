#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        log::debug!("[{}] {:?}", function_name!(), format_args!($($arg)+));
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        log::info!("[{}] {:?}", function_name!(), format_args!($($arg)+));
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        log::error!("[{}] {:?}", function_name!(), format_args!($($arg)+));
    }
}
