pub trait HashedPassword {
    fn sign_up(password: &str) -> String;
    fn sign_in(password: &str, password_hash: &str) -> bool;
}

#[cfg(not(target_arch = "wasm32"))]
pub type Auth = target::S;

#[cfg(not(target_arch = "wasm32"))]
mod target {
    use super::HashedPassword;
    use argon2::Argon2;
    use argon2::password_hash::PasswordHasher;
    use argon2::password_hash::PasswordVerifier;
    use argon2::password_hash::phc::PasswordHash;
    use derive_more::From;

    #[derive(Debug, Clone, From)]
    pub struct S;

    impl HashedPassword for S {
        fn sign_up(password: &str) -> String {
            Argon2::default()
                .hash_password(password.as_bytes())
                .unwrap()
                .to_string()
        }

        fn sign_in(password: &str, password_hash: &str) -> bool {
            let parsed_hash = PasswordHash::new(password_hash).unwrap();
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        }
    }
}
