#[macro_export]
macro_rules! ts_dbg {
    ($val:expr) => {{
        eprintln!("[{}]", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
        dbg!($val)
    }};
}

pub fn krx_tick_size(price: i64) -> i64 {
    if price <= 0 {
        return 1;
    }

    match price {
        0..=1_999 => 1,
        2_000..=4_999 => 5,
        5_000..=19_999 => 10,
        20_000..=49_999 => 50,
        50_000..=199_999 => 100,
        200_000..=499_999 => 500,
        _ => 1_000,
    }
}

#[cfg(test)]
mod tests {
    use super::krx_tick_size;

    #[test]
    fn krx_tick_size_boundaries() {
        assert_eq!(krx_tick_size(1), 1);
        assert_eq!(krx_tick_size(1_999), 1);
        assert_eq!(krx_tick_size(2_000), 5);
        assert_eq!(krx_tick_size(4_999), 5);
        assert_eq!(krx_tick_size(5_000), 10);
        assert_eq!(krx_tick_size(19_999), 10);
        assert_eq!(krx_tick_size(20_000), 50);
        assert_eq!(krx_tick_size(49_999), 50);
        assert_eq!(krx_tick_size(50_000), 100);
        assert_eq!(krx_tick_size(199_999), 100);
        assert_eq!(krx_tick_size(200_000), 500);
        assert_eq!(krx_tick_size(499_999), 500);
        assert_eq!(krx_tick_size(500_000), 1_000);
    }
}
