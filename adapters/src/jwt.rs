use crate::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
fn exp_time() -> u64 {
    (Utc::now() + ChronoDuration::minutes(30)).timestamp() as u64
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct Key {
    key: Arc<Vec<u8>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for Key {
    fn default() -> Self {
        Self {
            key: Arc::new("key".into()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
struct Claims {
    id: row_id::RowIdS,
    exp: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl JWT for Key {
    type UserId = row_id::RowIdS;
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
