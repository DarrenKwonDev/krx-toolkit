#[macro_export]
macro_rules! ts_dbg {
    ($val:expr) => {{
        eprintln!("[{}]", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
        dbg!($val)
    }};
}
