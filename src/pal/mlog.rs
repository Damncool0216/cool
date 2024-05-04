#[macro_export]
macro_rules! mdebug {
    ($($arg:tt)+) => {
        log::debug!("[{}] {:?}", function_name!(), format_args!($($arg)+));
    }
}

#[macro_export]
macro_rules! minfo {
    ($($arg:tt)+) => {
        log::info!("[{}] {:?}", function_name!(), format_args!($($arg)+));
    }
}

#[macro_export]
macro_rules! merror {
    ($($arg:tt)+) => {
        log::error!("[{}] {:?}", function_name!(), format_args!($($arg)+));
    }
}
