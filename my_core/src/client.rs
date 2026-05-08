use crate::prelude::*;

pub struct RoutsForClientSide<WA, RN, CH>
where
    WA: WAMP + 'static,
    RN: Runtime + 'static,
    CH: CacheIO + 'static,
{
    web_socket: WA,
    runtime: PhantomData<RN>,
    cache: CH,
}

impl<WA, RN, CH> RoutsForClientSide<WA, RN, CH>
where
    WA: WAMP + 'static,
    RN: Runtime + 'static,
    CH: CacheIO + 'static,
{
    pub async fn new(inner: WA, cache: CH) -> Self {
        Self {
            web_socket: inner,
            runtime: PhantomData::<RN>,
            cache,
        }
    }

    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let result = self
            .web_socket
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, HashimError>>(
                &sign_up::PATH.to_string(),
                input,
                100,
            )
            .await??;

        Ok(result)
    }

    pub async fn sign_in(&self, input: &sign_in::Input) -> Result<sign_in::Result, DynamicError> {
        let result = self
            .web_socket
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, HashimError>>(
                &sign_in::PATH.to_string(),
                input,
                100,
            )
            .await??;

        Ok(result)
    }

    pub async fn get_error(&self) -> DynamicError {
        self.web_socket.get_error().await
    }
    // TODO : i need to display the error to the ui
    // maybe i need to return rx "receiver" variabe to make it as actor model
    // pub fn data_receiver<Sg: Signal<T = String> + 'static>(self: Arc<Self>, err: Sg) {
    //     RN::spawn(async move {
    //         self.web_socket
    //             .receive_only::<data_receiver::Input>(
    //                 &data_receiver::PATH.to_string(),
    //                 async |data| {
    //                     let a = self.cache.write_data(&data).await;
    //                     match a {
    //                         Ok(_) => return,
    //                         Err(e) => err.set(e.to_string()),
    //                     };
    //                 },
    //             )
    //             .await;
    //     });
    // }
}
