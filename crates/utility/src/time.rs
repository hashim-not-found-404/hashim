pub trait Time: 'static {
    fn now_as_unix_milliseconds() -> u64;
}

#[cfg(feature = "infrastructure")]
pub mod target {
    use super::Time;

    pub struct S;

    impl Time for S {
        fn now_as_unix_milliseconds() -> u64 {
            chrono::Utc::now().timestamp_millis() as u64
        }
    }
}
