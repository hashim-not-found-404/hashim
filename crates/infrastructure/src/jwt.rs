use anyhow::Error;
use jsonwebtoken::decode_header;
use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct JsonWebTokenType(String);

impl JsonWebTokenType {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for JsonWebTokenType {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for JsonWebTokenType {
    type Error = Error;

    fn try_from(token: String) -> Result<Self, Self::Error> {
        decode_header(&token)?;
        Ok(JsonWebTokenType(token))
    }
}

pub trait JWT<Subject: Serialize + for<'de> Deserialize<'de> + Clone>: 'static {
    fn new() -> Self;
    fn sign(&self, subject: &Subject) -> JsonWebTokenType;
    fn validate(&self, token: JsonWebTokenType) -> Option<Subject>;
}

#[cfg(not(target_arch = "wasm32"))]
pub type Jwt = target::S;

#[cfg(not(target_arch = "wasm32"))]
mod target {
    use super::JWT;
    use super::JsonWebTokenType;
    use chrono::Duration;
    use chrono::Utc;
    use jsonwebtoken::Algorithm;
    use jsonwebtoken::DecodingKey;
    use jsonwebtoken::EncodingKey;
    use jsonwebtoken::Header;
    use jsonwebtoken::Validation;
    use jsonwebtoken::decode;
    use jsonwebtoken::encode;
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
    struct Claims<Subject> {
        id: Subject,
        exp: u64,
    }

    impl<Subject: Serialize + for<'de> Deserialize<'de> + Clone> JWT<Subject> for S {
        fn new() -> Self {
            Self {
                key: Arc::new("key".into()),
            }
        }

        fn sign(&self, subject: &Subject) -> JsonWebTokenType {
            let claims = Claims {
                id: subject.clone(),
                exp: exp_time(),
            };

            JsonWebTokenType(
                encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(&self.key),
                )
                .unwrap(),
            )
        }

        fn validate(&self, token: JsonWebTokenType) -> Option<Subject> {
            let result = decode::<Claims<Subject>>(
                &token.0,
                &DecodingKey::from_secret(&self.key),
                &Validation::new(Algorithm::HS256),
            );
            result.ok().map(|data| data.claims.id)
        }
    }
}
