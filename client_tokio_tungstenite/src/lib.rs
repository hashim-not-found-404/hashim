#[cfg(not(target_arch = "wasm32"))]
use agnostic_lite::tokio::TokioRuntime as NativeRuntime;

#[cfg(target_arch = "wasm32")]
use agnostic_lite::wasm::WasmRuntime as NativeRuntime;

use adapters::prelude::*;
use my_core::prelude::*;
use std::sync::Arc;

pub struct Dsdff(
    Arc<
        web_socket::MyClient<
            web_socket_adapter::MyClient,
            encode_decode::Atooooooooooo,
            random_number::RandomNumberS,
            NativeRuntime,
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
    type Error = MyError;

    async fn sign_up(&self, input: sign_up::Input) -> Result<sign_up::Result, Self::Error> {
        let result = self
            .0
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, ()>>(
                sign_up::PATH.to_string(),
                input,
                2,
            )
            .await;

        // let Ok(result) = NativeRuntime::timeout_local(Duration::from_secs(2), result).await else {
        //     return Err(MyError::Timeout);
        // };

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(MyError::ServerError),
            },
            Err(e) => Err(e),
        }
    }

    async fn sign_in(&self, input: sign_in::Input) -> Result<sign_in::Result, Self::Error> {
        let result = self
            .0
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, ()>>(
                sign_in::PATH.to_string(),
                input,
                2,
            )
            .await;

        // let Ok(result) = NativeRuntime::timeout_local(Duration::from_secs(2), result).await else {
        //     return Err(MyError::Timeout);
        // };

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(MyError::ServerError),
            },
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::{self, runtime, task};

    use super::*;

    #[tokio::test]
    async fn test_name() {
        let t = runtime::LocalRuntime::new().unwrap();
        t.spawn_blocking(async || {
            let a = Dsdff::new().await;
            a.sign_in(sign_in::Input {
                user_id: "hashim".to_string(),
                password: "1".to_string(),
            })
            .await
            .unwrap()
            .unwrap();
        });
    }
}
