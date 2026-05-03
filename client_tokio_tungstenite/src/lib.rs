use adapters::prelude::*;
use my_core::prelude::*;
use std::sync::Arc;

pub struct Dsdff(
    Arc<
        web_socket::MyClient<
            web_socket_adapter::m::S,
            encode_decode::m::S,
            random_number::m::S,
            runtime::m::S,
        >,
    >,
);

impl Dsdff {
    pub async fn new() -> Self {
        let url = format!("ws://{}/ws", ADDRESS);
        Self(web_socket::MyClient::connect(url.as_str()).await.unwrap())
    }
}

impl BackendRouts for Dsdff {
    async fn sign_up(&self, input: sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let result = self
            .0
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, ()>>(
                sign_up::PATH.to_string(),
                input,
                2,
            )
            .await;

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err("internal server error".into()),
            },
            Err(e) => return Err(e),
        }
    }

    async fn sign_in(&self, input: sign_in::Input) -> Result<sign_in::Result, DynamicError> {
        let result = self
            .0
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, ()>>(
                sign_in::PATH.to_string(),
                input,
                2,
            )
            .await;

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err("internal server error".into()),
            },
            Err(e) => return Err(e),
        }
    }
}
