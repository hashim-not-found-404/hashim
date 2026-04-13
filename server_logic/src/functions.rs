use derive_more::{Eq, From};
use my_core::traits;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use uuid::Uuid;

static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}]+(?:[-'][\p{L}\p{M}]+)*$").unwrap());

#[derive(Clone)]
pub struct Functions;

impl traits::Functions for Functions {
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
        test_suite::test_is_regex::<Functions>();
    }
}
