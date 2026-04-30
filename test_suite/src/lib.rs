use my_core::prelude::*;
use std::fmt::Debug;

pub fn test_hashed_password<T: HashedPassword + Clone>() {
    let password = "hashem".to_string();
    let hash = T::sign_up(password.clone());

    let is_ok = T::sign_in(password.clone(), hash.clone());
    assert_eq!(is_ok, true);

    let is_ok = T::sign_in("hosh".into(), hash.clone());
    assert_eq!(is_ok, false);
}

pub fn test_is_regex<T: Functions>() {
    let a = T::is_regex(&"hashem".to_string());
    assert_eq!(a, Ok(()));

    let a = T::is_regex(&"HASHEM".to_string());
    assert_eq!(a, Ok(()));

    let a = T::is_regex(&"hashem;sud".to_string());
    assert_ne!(a, Ok(()));

    let a = T::is_regex(&"*hashem./*-".to_string());
    assert_ne!(a, Ok(()));

    let a = T::is_regex(&"".to_string());
    assert_ne!(a, Ok(()));

    let a = T::is_regex(&"هاشم".to_string());
    assert_eq!(a, Ok(()));

    // Test cases
    let test_cases = vec![
        (true, "hello"),
        (true, "مرحبا"),        // Arabic
        (true, "привет"),       // Russian
        (true, "你好"),         // Chinese
        (true, "안녕하세요"),   // Korean
        (true, "こんにちは"),   // Japanese
        (true, "santé"),        // French with accent
        (true, "O'Connor"),     // With apostrophe
        (true, "well-known"),   // With hyphen
        (false, "hello123"),    // Contains numbers
        (false, "hello!"),      // Contains symbol
        (false, "hello world"), // Contains space
        (false, "😀"),          // Emoji
        (false, ""),            // Empty
        (true, "a-b-c"),        // Multiple hyphens
        (true, "l'heure"),      // French with apostrophe
    ];

    for (expected, text) in test_cases {
        let result = T::is_regex(&text.to_string());
        assert_eq!(expected == true && result.is_err(), false);
    }
}

pub fn test_jwt<Jwt, Id>()
where
    Jwt: JWT<UserId = Id> + Default + Debug,
    Id: RowId + Eq + Debug,
{
    let jwt = Jwt::default();
    let input_id = Id::generate();

    let token = jwt.sign(&input_id);
    let output_id = jwt.validate(token).unwrap();

    assert_eq!(output_id, input_id);
}

pub async fn test_commit<DB, Id, H>()
where
    DB: Database + Default,
    for<'a> <DB::Client as DBClient>::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = H>,
    Id: RowId,
    H: HashedPassword,
    // Add these Debug bounds:
    <DB as Database>::Error: Debug,
    <<DB as Database>::Client as DBClient>::Error: Debug,
    for<'a> <<<DB as Database>::Client as DBClient>::Txn<'a> as DBTransaction>::Error: Debug,
{
    let db = DB::default();
    let mut client = db.get_client().await.unwrap();
    let txn = client.begin_transaction().await.unwrap();

    txn.commit_transaction().await.unwrap().unwrap();
}
