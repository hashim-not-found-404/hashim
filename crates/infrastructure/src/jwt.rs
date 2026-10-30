use anyhow::Error;
use jwt_simple::prelude::*;
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
        Token::decode_metadata(&token)?;
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
    use jwt_simple::prelude::*;
    use serde::Deserialize;
    use serde::Serialize;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Debug, Deserialize, Serialize)]
    struct CustomClaims<Subject> {
        id: Subject,
    }

    #[derive(Debug, Clone)]
    pub struct S {
        key: Arc<HS256Key>,
    }

    fn exp_duration() -> Duration {
        Duration::from_secs(30 * 60)
    }

    impl<Subject: Serialize + for<'de> Deserialize<'de> + Clone> JWT<Subject> for S {
        fn new() -> Self {
            let key_bytes = b"keydflzjlzfjlzjflzfzvzjzssjdofzfii";
            let key = HS256Key::from_bytes(key_bytes);
            Self { key: Arc::new(key) }
        }

        fn sign(&self, subject: &Subject) -> JsonWebTokenType {
            let custom = CustomClaims {
                id: subject.clone(),
            };
            let claims = Claims::with_custom_claims(custom, exp_duration().into());
            let token = self.key.authenticate(claims).unwrap();
            JsonWebTokenType(token)
        }

        fn validate(&self, token: JsonWebTokenType) -> Option<Subject> {
            let claims = self
                .key
                .verify_token::<CustomClaims<Subject>>(&token.0, None)
                .ok()?;
            Some(claims.custom.id)
        }
    }
}
