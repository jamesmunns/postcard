#![cfg(feature = "log-v0_4")]

#[test]
fn log_level_smoke() {
    for level in [
        log_v0_4::Level::Error,
        log_v0_4::Level::Warn,
        log_v0_4::Level::Info,
        log_v0_4::Level::Debug,
        log_v0_4::Level::Trace,
    ] {
        super::round_trip_test(level);
    }
}

#[test]
fn log_level_filter_smoke() {
    for level_filter in [
        log_v0_4::LevelFilter::Off,
        log_v0_4::LevelFilter::Error,
        log_v0_4::LevelFilter::Warn,
        log_v0_4::LevelFilter::Info,
        log_v0_4::LevelFilter::Debug,
        log_v0_4::LevelFilter::Trace,
    ] {
        super::round_trip_test(level_filter);
    }
}
