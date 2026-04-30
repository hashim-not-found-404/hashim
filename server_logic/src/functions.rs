use my_core::prelude::*;
use regex::Regex;
use std::sync::LazyLock;

static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}]+(?:[-'][\p{L}\p{M}]+)*$").unwrap());

#[derive(Clone)]
pub struct FunctionsS;

impl Functions for FunctionsS {
    fn is_regex(s: &String) -> Result<(), String> {
        match RE.is_match(s) {
            true => return Ok(()),
            false => return Err("not match".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_regex() {
        test_suite::test_is_regex::<FunctionsS>();
    }
}
