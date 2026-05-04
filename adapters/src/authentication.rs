#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;
    use argon2::{
        Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
        password_hash::{SaltString, rand_core::OsRng},
    };
    use derive_more::From;

    #[derive(Clone, From)]
    pub struct S(String);

    impl S {
        pub fn into_inner(&self) -> String {
            self.0.clone()
        }
    }

    impl HashedPassword for S {
        fn sign_up(password: &String) -> Self {
            // 1. Generate a cryptographically random salt
            let salt = SaltString::generate(&mut OsRng);

            // 2. Configure Argon2 (default uses secure parameters)
            let argon2 = Argon2::default();

            // 3. Hash the password into a PHC string (e.g., "$argon2id$v=19$...")
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            // 4. You need to return or store this string.
            // You'll have to convert `password_hash` (String) into your `TableUserFieldPass` type.
            Self(password_hash)
        }

        fn sign_in(password: &String, password_hash: &Self) -> bool {
            // 1. Parse the stored PHC string
            let parsed_hash = PasswordHash::new(&password_hash.0).unwrap();

            // 2. Verify the provided password against the parsed hash
            // The verification uses the parameters (salt, cost, etc.) embedded in the parsed hash.
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        }
    }
}
