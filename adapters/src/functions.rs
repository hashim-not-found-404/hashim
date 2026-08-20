pub mod target {
    use my_core::utility::traits::Regex as MyRegex;
    use regex::Regex;
    use std::sync::LazyLock;

    #[derive(Debug, Clone)]
    pub struct S;

    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}]+(?:[-'][\p{L}\p{M}]+)*$").unwrap());

    impl MyRegex for S {
        fn is_regex(s: &str) -> Result<(), String> {
            match RE.is_match(s) {
                true => Ok(()),
                false => Err("not match".to_string()),
            }
        }
    }
}
