use crate::prelude::*;

pub trait CacheIO: Sized {
    async fn new() -> Result<Self, DynamicError>;
    async fn write_data(&self, data: &data_receiver::Input) -> Result<(), DynamicError>;
}

pub struct RoutsForClientSide<WA, RN, CH>
where
    WA: WAMP + 'static,
    RN: Runtime + 'static,
    CH: CacheIO + 'static,
{
    web_socket: Arc<WA>,
    runtime: PhantomData<RN>,
    cache: CH,
}

impl<WA, RN, CH> RoutsForClientSide<WA, RN, CH>
where
    WA: WAMP + 'static,
    RN: Runtime + 'static,
    CH: CacheIO + 'static,
{
    pub async fn new(inner: Arc<WA>, cache: CH) -> Self {
        Self {
            web_socket: inner,
            runtime: PhantomData::<RN>,
            cache,
        }
    }

    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let result = self
            .web_socket
            .clone()
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
            .web_socket
            .clone()
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, HashimError>>(
                &sign_in::PATH.to_string(),
                input,
                2,
            )
            .await??;

        Ok(result)
    }

    pub fn data_receiver<Sg: Signal<T = String> + 'static>(self: Arc<Self>, err: Sg) {
        RN::spawn(async move {
            self.web_socket
                .clone()
                .receive_only::<data_receiver::Input>(
                    &data_receiver::PATH.to_string(),
                    async |data| {
                        let a = self.cache.write_data(&data).await;
                        match a {
                            Ok(_) => return,
                            Err(e) => err.set(e.to_string()),
                        };
                    },
                )
                .await;
        });
    }
}
