use crate::prelude::*;

pub struct RoutsForClientSide<T: WebSocket> {
    inner: Arc<T>,
}

impl<T: WebSocket> RoutsForClientSide<T> {
    pub async fn new(inner: Arc<T>) -> Self {
        Self { inner }
    }
}

impl<T: WebSocket> RoutsForClientSide<T> {
    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let result = self
            .inner
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, HashimError>>(
                &sign_up::PATH.to_string(),
                input,
                2,
            )
            .await??;

        Ok(result)
    }

    pub async fn sign_in(&self, input: &sign_in::Input) -> Result<sign_in::Result, DynamicError> {
        let result = self
            .inner
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, HashimError>>(
                &sign_in::PATH.to_string(),
                input,
                2,
            )
            .await??;

        Ok(result)
    }
}
