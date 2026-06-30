pub mod m {
    use getrandom::fill;
    use my_core::prelude::*;

    pub struct S;

    impl RandomNumber for S {
        fn generate() -> u64 {
            let mut buf = [0u8; 8];
            fill(&mut buf).unwrap();
            u64::from_ne_bytes(buf)
        }
    }
}
