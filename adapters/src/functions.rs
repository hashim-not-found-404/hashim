#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;
    use regex::Regex;
    use std::sync::LazyLock;

    #[derive(Clone)]
    pub struct S;

    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}]+(?:[-'][\p{L}\p{M}]+)*$").unwrap());

    impl Functions for S {
        fn is_regex(s: &String) -> Result<(), String> {
            match RE.is_match(s) {
                true => return Ok(()),
                false => return Err("not match".to_string()),
            }
        }
    }
}
