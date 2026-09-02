pub mod target {
    use my_core::utility::traits::Time;

    pub struct S;

    impl Time for S {
        fn now_as_unix_milliseconds() -> u64 {
            chrono::Utc::now().timestamp_millis() as u64
        }
    }
}
