use crate::prelude::*;

const TIMEOUT: u32 = 5;

pub struct RoutsForClientSide<WA, RT, MPSC, CH>
where
    WA: WAMP + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    CH: CacheIO + 'static,
{
    web_socket: WA,
    runtime: PhantomData<RT>,
    mpsc: PhantomData<MPSC>,
    cache: CH,
}

impl<WA, RT, MPSC, CH> RoutsForClientSide<WA, RT, MPSC, CH>
where
    WA: WAMP<Sender<DynamicError> = MPSC::Sender<DynamicError>> + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    CH: CacheIO + 'static,
{
    pub async fn new(sender_to_error: MPSC::Sender<DynamicError>) -> Arc<Self> {
        let url = format!("ws://{}/ws", ADDRESS);
        let web_socket = WA::new(sender_to_error.clone());
        web_socket.connect_to_url(&url).await;

        let routs_for_client_side = Arc::new(Self {
            web_socket,
            runtime: PhantomData,
            mpsc: PhantomData,
            cache: CH::new().await.unwrap(),
        });

        routs_for_client_side
            .clone()
            .data_receiver(sender_to_error)
            .await;

        routs_for_client_side
    }

    pub async fn sign_up(
        self: Arc<Self>,
        input: &sign_up::Input,
    ) -> Result<sign_up::Result, DynamicError> {
        let result = self
            .web_socket
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, HashimError>>(
                &sign_up::PATH.to_string(),
                input,
                TIMEOUT,
            )
            .await??;

        Ok(result)
    }

    pub async fn sign_in(
        self: Arc<Self>,
        input: &sign_in::Input,
    ) -> Result<sign_in::Result, DynamicError> {
        let result = self
            .web_socket
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, HashimError>>(
                &sign_in::PATH.to_string(),
                input,
                TIMEOUT,
            )
            .await??;

        Ok(result)
    }

    pub async fn create_company(
        self: Arc<Self>,
        input: &create_company::Input,
    ) -> Result<create_company::Result, DynamicError> {
        let result = self
            .web_socket
            .send_and_receive::<create_company::Input, Result<create_company::Result, HashimError>>(
                &create_company::PATH.to_string(),
                input,
                TIMEOUT,
            )
            .await??;

        Ok(result)
    }

    pub async fn create_company_branch(
        self: Arc<Self>,
        input: &create_company_branch::Input,
    ) -> Result<create_company_branch::Result, DynamicError> {
        let result = self
            .web_socket
            .send_and_receive::<create_company_branch::Input, Result<create_company_branch::Result, HashimError>>(
                &create_company_branch::PATH.to_string(),
                input,
                TIMEOUT,
            )
            .await??;

        Ok(result)
    }

    pub async fn data_receiver(self: Arc<Self>, sender_to_error: MPSC::Sender<DynamicError>) {
        self.clone()
            .web_socket
            .receive_only::<data_receiver::Input>(
                &data_receiver::PATH.to_string(),
                async move |data| {
                    let a = self.cache.write_data(&data).await;
                    match a {
                        Ok(_) => return,
                        Err(e) => sender_to_error.send(e).await.unwrap(),
                    };
                },
            )
            .await;
    }
}
