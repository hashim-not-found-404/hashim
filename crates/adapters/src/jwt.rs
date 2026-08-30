#[cfg(not(target_arch = "wasm32"))]
pub mod target {
    use chrono::Duration;
    use chrono::Utc;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::DecodingKey;
    use jsonwebtoken::EncodingKey;
    use jsonwebtoken::Header;
    use jsonwebtoken::Validation;
    use jsonwebtoken::decode;
    use jsonwebtoken::encode;
    use my_core::domain::utility::new_types::UserUuid;
    use my_core::domain::utility::types::JWT;
    use my_core::domain::utility::types::JsonWebTokenType;
    use serde::Deserialize;
    use serde::Serialize;
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    pub struct S {
        key: Arc<Vec<u8>>,
    }

    fn exp_time() -> u64 {
        (Utc::now() + Duration::minutes(30)).timestamp() as u64
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct Claims {
        id:  UserUuid,
        exp: u64,
    }

    impl JWT for S {
        fn new() -> Self {
            Self {
                key: Arc::new("key".into()),
            }
        }

        fn sign(&self, id: &UserUuid) -> JsonWebTokenType {
            let claims = Claims {
                id:  id.clone(),
                exp: exp_time(),
            };

            JsonWebTokenType(
                encode(&Header::default(), &claims, &EncodingKey::from_secret(&self.key)).unwrap(),
            )
        }

        fn validate(&self, token: JsonWebTokenType) -> Option<UserUuid> {
            let result = decode::<Claims>(
                &token.0,
                &DecodingKey::from_secret(&self.key),
                &Validation::new(Algorithm::HS256),
            );

            match result {
                Ok(token) => Some(token.claims.id),
                Err(_) => None,
            }
        }
    }
}
