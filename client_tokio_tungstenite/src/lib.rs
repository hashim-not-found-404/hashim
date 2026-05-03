use adapters::prelude::*;
use my_core::prelude::*;
use std::sync::Arc;

pub struct Dsdff(
    Arc<
        web_socket::MyClient<
            web_socket_adapter::MyClient,
            encode_decode::Atooooooooooo,
            random_number::RandomNumberS,
            runtime::RuntimeS,
        >,
    >,
);

impl Dsdff {
    pub async fn new() -> Self {
        Self(web_socket::MyClient::new(
            web_socket_adapter::MyClient::new().await,
        ))
    }
}

impl BackendRouts for Dsdff {
    async fn sign_up(&self, input: sign_up::Input) -> Result<sign_up::Result, ThisISTheNewError> {
        let result = self
            .0
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, ()>>(
                sign_up::PATH.to_string(),
                input,
                20,
            )
            .await;

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(Box::new(MyError::ServerError)),
            },
            Err(e) => return Err(e),
        }
    }

    async fn sign_in(&self, input: sign_in::Input) -> Result<sign_in::Result, ThisISTheNewError> {
        let result = self
            .0
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, ()>>(
                sign_in::PATH.to_string(),
                input,
                20,
            )
            .await;

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(Box::new(MyError::ServerError)),
            },
            Err(e) => return Err(e),
        }
    }
}
