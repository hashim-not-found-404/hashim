pub(crate) trait Searchable {
    fn search_key(&self) -> String;
}

pub(crate) fn select_strings<T: Searchable>(s_list: Vec<T>, s: String) -> Vec<T> {
    if s.is_empty() {
        return s_list;
    }
    let needle = s.to_lowercase();
    s_list
        .into_iter()
        .filter(|item| is_subsequence(&needle, &item.search_key().to_lowercase()))
        .collect()
}

/// Returns `true` if all characters of `needle` appear in `haystack` in order.
fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    for c in needle.chars() {
        loop {
            match chars.next() {
                Some(ch) if ch == c => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Searchable for String {
        fn search_key(&self) -> String {
            self.clone()
        }
    }
    #[test]
    fn fuzzy_search() {
        let list = vec!["apple".to_string(), "banana".to_string()];
        let result = select_strings(list.clone(), "apl".to_string());
        assert_eq!(result, vec!["apple".to_string()]);

        let result = select_strings(list.clone(), "bnn".to_string());
        assert_eq!(result, vec!["banana".to_string()]);

        let result = select_strings(list.clone(), "aa".to_string());
        assert_eq!(result, vec!["banana".to_string()]);

        let result = select_strings(list.clone(), "ab".to_string());
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn empty_search_returns_all() {
        let list = vec!["apple".to_string(), "banana".to_string()];
        let result = select_strings(list.clone(), "".to_string());
        assert_eq!(result, list);
    }

    #[test]
    fn empty_list_returns_empty() {
        let list: Vec<String> = vec![];
        let result = select_strings(list, "a".to_string());
        assert!(result.is_empty());
    }

    #[test]
    fn exact_match_returns_one() {
        let list = vec!["apple".to_string(), "banana".to_string()];
        let result = select_strings(list, "apple".to_string());
        assert_eq!(result, vec!["apple".to_string()]);
    }

    #[test]
    fn substring_match() {
        let list = vec!["apple".to_string(), "pineapple".to_string(), "banana".to_string()];
        let result = select_strings(list, "app".to_string());
        assert_eq!(result, vec!["apple".to_string(), "pineapple".to_string()]);
    }

    #[test]
    fn case_insensitive() {
        let list = vec!["Apple".to_string(), "BANANA".to_string(), "Grape".to_string()];
        let result = select_strings(list, "ap".to_string());
        assert_eq!(result, vec!["Apple".to_string(), "Grape".to_string()]); // "Apple" contains "ap" case‑insensitively

        let list2 = vec!["Apple".to_string(), "BANANA".to_string(), "Grape".to_string()];
        let result2 = select_strings(list2, "ban".to_string());
        assert_eq!(result2, vec!["BANANA".to_string()]);
    }

    #[test]
    fn no_match_returns_empty() {
        let list = vec!["apple".to_string(), "banana".to_string()];
        let result = select_strings(list, "xyz".to_string());
        assert!(result.is_empty());
    }

    #[test]
    fn handles_unicode_characters() {
        let list = vec!["café".to_string(), "coffee".to_string(), "tea".to_string()];
        let result = select_strings(list, "é".to_string());
        assert_eq!(result, vec!["café".to_string()]);
    }

    #[test]
    fn does_not_modify_original_list() {
        // If the function consumes ownership, you can't check the original, but we test that
        // the returned vector is a new one (not a reference to the original).
        let original = vec!["one".to_string(), "two".to_string()];
        let result = select_strings(original.clone(), "o".to_string());
        assert_eq!(result, vec!["one".to_string(), "two".to_string()]);
        // The original is still intact (if we cloned)
        assert_eq!(original, vec!["one".to_string(), "two".to_string()]);
    }
}
