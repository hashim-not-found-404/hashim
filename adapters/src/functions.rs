pub mod m {
    use crate::prelude::Regex as MyRegex;
    use regex::Regex;
    use std::sync::LazyLock;

    #[derive(Clone)]
    pub struct S;

    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}]+(?:[-'][\p{L}\p{M}]+)*$").unwrap());

    impl MyRegex for S {
        fn is_regex(s: &String) -> Result<(), String> {
            match RE.is_match(s) {
                true => Ok(()),
                false => Err("not match".to_string()),
            }
        }
    }
}
