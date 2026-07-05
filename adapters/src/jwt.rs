#[cfg(not(target_arch = "wasm32"))]
pub mod target {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
    use my_core::accounting_domain::db_types;
    use my_core::accounting_domain::decider::JWT;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    pub struct S {
        key: Arc<Vec<u8>>,
    }

    fn exp_time() -> u64 {
        (Utc::now() + Duration::minutes(30)).timestamp() as u64
    }

    #[derive(Serialize, Deserialize)]
    struct Claims {
        id: db_types::UuidType,
        exp: u64,
    }

    impl JWT for S {
        fn new() -> Self {
            Self {
                key: Arc::new("key".into()),
            }
        }

        fn sign(&self, id: &db_types::UuidType) -> db_types::JsonWebTokenType {
            let claims = Claims {
                id: id.clone(),
                exp: exp_time(),
            };

            db_types::JsonWebTokenType(
                encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(&self.key),
                )
                .unwrap(),
            )
        }

        fn validate(&self, token: db_types::JsonWebTokenType) -> Option<db_types::UuidType> {
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
