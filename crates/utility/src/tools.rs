pub fn select_strings<T>(s_list: Vec<T>, s: impl AsRef<str>, search_key: fn(&T) -> &str) -> Vec<T> {
    if s.as_ref().is_empty() {
        return s_list;
    }
    let needle = s.as_ref().to_lowercase();
    s_list
        .into_iter()
        .filter(|item| is_subsequence(&needle, search_key(item).to_lowercase().as_str()))
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    for c in needle.chars() {
        loop {
            match chars.next() {
                Some(ch) if ch == c => break,
                Some(_) => {}
                None => return false,
            }
        }
    }
    true
}

#[cfg(test)]
mod tests_select_strings {
    use super::*;

    fn search_key(s: &str) -> &str {
        s
    }

    #[test]
    fn fuzzy_search() {
        let list = vec!["apple", "banana"];
        let result = select_strings(list.clone(), "apl", |a| search_key(a));
        assert_eq!(result, vec!["apple"]);

        let result = select_strings(list.clone(), "bnn", |a| search_key(a));
        assert_eq!(result, vec!["banana"]);

        let result = select_strings(list.clone(), "aa", |a| search_key(a));
        assert_eq!(result, vec!["banana"]);

        let result = select_strings(list.clone(), "ab", |a| search_key(a));
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn empty_search_returns_all() {
        let list = vec!["apple", "banana"];
        let result = select_strings(list.clone(), "", |a| search_key(a));
        assert_eq!(result, list);
    }

    #[test]
    fn empty_list_returns_empty() {
        let list: Vec<&str> = vec![];
        let result = select_strings(list, "a", |a| search_key(a));
        assert!(result.is_empty());
    }

    #[test]
    fn exact_match_returns_one() {
        let list = vec!["apple", "banana"];
        let result = select_strings(list, "apple", |a| search_key(a));
        assert_eq!(result, vec!["apple"]);
    }

    #[test]
    fn substring_match() {
        let list = vec!["apple", "pineapple", "banana"];
        let result = select_strings(list, "app", |a| search_key(a));
        assert_eq!(result, vec!["apple", "pineapple"]);
    }

    #[test]
    fn case_insensitive() {
        let list = vec!["Apple", "BANANA", "Grape"];
        let result = select_strings(list, "ap", |a| search_key(a));
        assert_eq!(result, vec!["Apple", "Grape"]);

        let list2 = vec!["Apple", "BANANA", "Grape"];
        let result2 = select_strings(list2, "ban", |a| search_key(a));
        assert_eq!(result2, vec!["BANANA"]);
    }

    #[test]
    fn no_match_returns_empty() {
        let list = vec!["apple", "banana"];
        let result = select_strings(list, "xyz", |a| search_key(a));
        assert!(result.is_empty());
    }

    #[test]
    fn handles_unicode_characters() {
        let list = vec!["café", "coffee", "tea"];
        let result = select_strings(list, "é", |a| search_key(a));
        assert_eq!(result, vec!["café"]);
    }

    #[test]
    fn does_not_modify_original_list() {
        let original = vec!["one", "two"];
        let result = select_strings(original.clone(), "o", |a| search_key(a));
        assert_eq!(result, vec!["one", "two"]);
        assert_eq!(original, vec!["one", "two"]);
    }
}

pub trait Sortable {
    type Key: Ord;
    fn key(&self) -> Self::Key;
}

pub fn sort<T: Sortable>(list: &mut Vec<T>) -> &Vec<T> {
    list.sort_by_key(T::key);
    list
}

#[cfg(test)]
mod tests_sort {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    impl Sortable for Person {
        type Key = (u32, String);

        fn key(&self) -> Self::Key {
            (self.age, self.name.clone())
        }
    }

    #[derive(Debug, PartialEq)]
    struct Product {
        id: u32,
        price: f64,
    }

    impl Sortable for Product {
        type Key = u32;

        fn key(&self) -> Self::Key {
            self.id
        }
    }

    #[test]
    fn sort_empty_list() {
        let mut list: Vec<Person> = vec![];
        let result = sort(&mut list);
        assert!(result.is_empty());
    }

    #[test]
    fn sort_single_element() {
        let mut list = vec![Person {
            name: "Alice".to_string(),
            age: 30,
        }];
        let result = sort(&mut list);
        assert_eq!(
            result,
            &vec![Person {
                name: "Alice".to_string(),
                age: 30,
            }]
        );
    }

    #[test]
    fn sort_by_age_then_name() {
        let mut list = vec![
            Person {
                name: "Bob".to_string(),
                age: 25,
            },
            Person {
                name: "Alice".to_string(),
                age: 30,
            },
            Person {
                name: "Charlie".to_string(),
                age: 25,
            },
        ];
        let expected = vec![
            Person {
                name: "Bob".to_string(),
                age: 25,
            },
            Person {
                name: "Charlie".to_string(),
                age: 25,
            },
            Person {
                name: "Alice".to_string(),
                age: 30,
            },
        ];
        let result = sort(&mut list);
        assert_eq!(result, &expected);
    }

    #[test]
    fn sort_with_primitive_key() {
        let mut list = vec![
            Product { id: 3, price: 10.0 },
            Product { id: 1, price: 20.0 },
            Product { id: 2, price: 15.0 },
        ];
        let expected = vec![
            Product { id: 1, price: 20.0 },
            Product { id: 2, price: 15.0 },
            Product { id: 3, price: 10.0 },
        ];
        let result = sort(&mut list);
        assert_eq!(result, &expected);
    }

    #[test]
    fn sort_is_not_stable() {
        #[derive(Debug, PartialEq)]
        struct EqualKey {
            id: u32,
            value: char,
        }
        impl Sortable for EqualKey {
            type Key = u32;

            fn key(&self) -> Self::Key {
                self.id
            }
        }
        let mut list = vec![
            EqualKey { id: 1, value: 'a' },
            EqualKey { id: 2, value: 'b' },
            EqualKey { id: 1, value: 'c' },
        ];
        let result = sort(&mut list);
        let ids: Vec<_> = result.iter().map(|e| e.id).collect();
        assert_eq!(ids, vec![1, 1, 2]);
    }

    #[test]
    fn sort_multiple_key() {
        #[derive(Debug, PartialEq)]
        struct EqualKey {
            id: u32,
            value: char,
        }
        impl Sortable for EqualKey {
            type Key = (char, u32);

            fn key(&self) -> Self::Key {
                (self.value, self.id)
            }
        }
        let mut list = vec![
            EqualKey { id: 1, value: 'a' },
            EqualKey { id: 2, value: 'b' },
            EqualKey { id: 1, value: 'c' },
            EqualKey { id: 0, value: 'c' },
        ];
        let result = sort(&mut list);
        let expected = vec![
            EqualKey { id: 1, value: 'a' },
            EqualKey { id: 2, value: 'b' },
            EqualKey { id: 0, value: 'c' },
            EqualKey { id: 1, value: 'c' },
        ];
        assert_eq!(result, &expected);
    }
}
