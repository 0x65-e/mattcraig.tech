use cfg_if::cfg_if;
use worker::console_log;
use worker::Request;
use worker::Date;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

pub fn log_not_present_error(kv: &str, key: &str) {
    console_log!(
        "{} - [{}], key \"{}\" not present in store",
        Date::now().to_string(),
        kv,
        key
    );
}

pub fn log_invalid_filename(key: &str) {
    console_log!(
        "{} - [{}], requested page \"{}\" uses non-UTF-8 characters",
        Date::now().to_string(),
        key,
        key
    );
}

pub fn log_generic_error(key: &str, err: &str) {
    console_log!(
        "{} - [{}], received generic worker error: {}",
        Date::now().to_string(),
        key,
        err
    )
}
