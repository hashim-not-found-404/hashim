use crate::prelude::*;

pub struct RandomNumberS;
impl RandomNumber for RandomNumberS {
    fn generate() -> u64 {
        let mut buf = [0u8; 8];
        fill(&mut buf).unwrap();
        u64::from_ne_bytes(buf)
    }
}
