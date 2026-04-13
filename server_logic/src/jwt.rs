use chrono::{Duration, Utc};
use impls_for_wasm::a1::RowId;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use my_core::traits;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn exp_time() -> u64 {
    (Utc::now() + Duration::minutes(30)).timestamp() as u64
}

#[derive(Clone, Debug)]
pub struct Key {
    key: Arc<Vec<u8>>,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            key: Arc::new("key".into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Claims {
    id: RowId,
    exp: u64,
}

impl traits::JWT for Key {
    type UserId = RowId;
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

    fn validate(&self, token: Self::JsonWebToken) -> Result<Self::UserId, ()> {
        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(&self.key),
            &Validation::new(Algorithm::HS256),
        );

        match result {
            Ok(token) => return Ok(token.claims.id),
            Err(_) => return Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt() {
        test_suite::test_jwt::<Key, RowId>();
    }
}
