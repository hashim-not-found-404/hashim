pub(crate) trait Searchable {
    fn search_key(&self) -> String;
}

pub(crate) fn select_strings<T: Searchable>(s_list: Vec<T>, s: impl AsRef<str>) -> Vec<T> {
    if s.as_ref().is_empty() {
        return s_list;
    }
    let needle = s.as_ref().to_lowercase();
    s_list
        .into_iter()
        .filter(|item| is_subsequence(&needle, &item.search_key().to_lowercase()))
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
        assert_eq!(result, vec!["Apple".to_string(), "Grape".to_string()]);

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
        let original = vec!["one".to_string(), "two".to_string()];
        let result = select_strings(original.clone(), "o".to_string());
        assert_eq!(result, vec!["one".to_string(), "two".to_string()]);
        assert_eq!(original, vec!["one".to_string(), "two".to_string()]);
    }
}

pub(crate) trait Sortable {
    type Key: Ord;
    fn key(&self) -> Self::Key;
}

pub(crate) fn sort<T: Sortable>(list: &mut Vec<T>) -> &Vec<T> {
    list.sort_by_key(T::key);
    list
}

#[cfg(test)]
mod tests_sort {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Person {
        name: String,
        age:  u32,
    }

    impl Sortable for Person {
        type Key = (u32, String);

        fn key(&self) -> Self::Key {
            (self.age, self.name.clone())
        }
    }

    #[derive(Debug, PartialEq)]
    struct Product {
        id:    u32,
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
            age:  30,
        }];
        let result = sort(&mut list);
        assert_eq!(result, &vec![Person {
            name: "Alice".to_string(),
            age:  30,
        }]);
    }

    #[test]
    fn sort_by_age_then_name() {
        let mut list = vec![
            Person {
                name: "Bob".to_string(),
                age:  25,
            },
            Person {
                name: "Alice".to_string(),
                age:  30,
            },
            Person {
                name: "Charlie".to_string(),
                age:  25,
            },
        ];
        let expected = vec![
            Person {
                name: "Bob".to_string(),
                age:  25,
            },
            Person {
                name: "Charlie".to_string(),
                age:  25,
            },
            Person {
                name: "Alice".to_string(),
                age:  30,
            },
        ];
        let result = sort(&mut list);
        assert_eq!(result, &expected);
    }

    #[test]
    fn sort_with_primitive_key() {
        let mut list = vec![
            Product {
                id:    3,
                price: 10.0,
            },
            Product {
                id:    1,
                price: 20.0,
            },
            Product {
                id:    2,
                price: 15.0,
            },
        ];
        let expected = vec![
            Product {
                id:    1,
                price: 20.0,
            },
            Product {
                id:    2,
                price: 15.0,
            },
            Product {
                id:    3,
                price: 10.0,
            },
        ];
        let result = sort(&mut list);
        assert_eq!(result, &expected);
    }

    #[test]
    fn sort_is_not_stable() {
        #[derive(Debug, PartialEq)]
        struct EqualKey {
            id:    u32,
            value: char,
        }
        impl Sortable for EqualKey {
            type Key = u32;

            fn key(&self) -> Self::Key {
                self.id
            }
        }
        let mut list = vec![
            EqualKey {
                id:    1,
                value: 'a',
            },
            EqualKey {
                id:    2,
                value: 'b',
            },
            EqualKey {
                id:    1,
                value: 'c',
            },
        ];
        let result = sort(&mut list);
        let ids: Vec<_> = result.iter().map(|e| e.id).collect();
        assert_eq!(ids, vec![1, 1, 2]);
    }

    #[test]
    fn sort_multiple_key() {
        #[derive(Debug, PartialEq)]
        struct EqualKey {
            id:    u32,
            value: char,
        }
        impl Sortable for EqualKey {
            type Key = (char, u32);

            fn key(&self) -> Self::Key {
                (self.value, self.id)
            }
        }
        let mut list = vec![
            EqualKey {
                id:    1,
                value: 'a',
            },
            EqualKey {
                id:    2,
                value: 'b',
            },
            EqualKey {
                id:    1,
                value: 'c',
            },
            EqualKey {
                id:    0,
                value: 'c',
            },
        ];
        let result = sort(&mut list);
        let expected = vec![
            EqualKey {
                id:    1,
                value: 'a',
            },
            EqualKey {
                id:    2,
                value: 'b',
            },
            EqualKey {
                id:    0,
                value: 'c',
            },
            EqualKey {
                id:    1,
                value: 'c',
            },
        ];
        assert_eq!(result, &expected);
    }
}
