use crate::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}]+(?:[-'][\p{L}\p{M}]+)*$").unwrap());

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct FunctionsS;

#[cfg(not(target_arch = "wasm32"))]
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
