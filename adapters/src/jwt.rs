#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;

    fn exp_time() -> u64 {
        (Utc::now() + ChronoDuration::minutes(30)).timestamp() as u64
    }

    #[derive(Clone, Debug)]
    pub struct S {
        key: Arc<Vec<u8>>,
    }

    impl Default for S {
        fn default() -> Self {
            Self {
                key: Arc::new("key".into()),
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Claims {
        id: row_id::m::S,
        exp: u64,
    }

    impl JWT for S {
        type UserId = row_id::m::S;
        type JsonWebToken = String;

        fn sign(&self, id: &Self::UserId) -> Self::JsonWebToken {
            let claims = Claims {
                id: id.clone(),
                exp: exp_time(),
            };

            encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(&self.key),
            )
            .unwrap()
            .into()
        }

        fn validate(&self, token: Self::JsonWebToken) -> Option<Self::UserId> {
            let result = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(&self.key),
                &Validation::new(Algorithm::HS256),
            );

            match result {
                Ok(token) => return Some(token.claims.id),
                Err(_) => return None,
            }
        }
    }
}
