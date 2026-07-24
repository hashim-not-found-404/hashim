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
    use my_core::accounting_domain::utility::types;
    use my_core::accounting_domain::utility::types::JWT;
    use serde::Deserialize;
    use serde::Serialize;
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
        id:  types::UuidType,
        exp: u64,
    }

    impl JWT for S {
        fn new() -> Self {
            Self {
                key: Arc::new("key".into()),
            }
        }

        fn sign(&self, id: &types::UuidType) -> types::JsonWebTokenType {
            let claims = Claims {
                id:  id.clone(),
                exp: exp_time(),
            };

            types::JsonWebTokenType(
                encode(&Header::default(), &claims, &EncodingKey::from_secret(&self.key)).unwrap(),
            )
        }

        fn validate(&self, token: types::JsonWebTokenType) -> Option<types::UuidType> {
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
