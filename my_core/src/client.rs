use crate::prelude::*;

const TIMEOUT: u32 = 5;

pub struct RoutsForClientSide<WS, DE, RN, RT, CH, Id, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    CH: CacheIO + 'static,
    Id: RowId + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    _ph: PhantomData<(WS, DE, RN, RT, CH, Id, MPSC)>,
    my_wamp: web_socket::MyWAMP<WS, DE, RN, RT, CH, Id, MPSC>,
}

impl<WS, DE, RN, RT, CH, Id, MPSC> RoutsForClientSide<WS, DE, RN, RT, CH, Id, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    CH: CacheIO + 'static,
    Id: RowId + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    pub async fn new(sender_to_error: MPSC::Sender<DynamicError>) -> Self {
        let web_socket =
            web_socket::MyWAMP::<WS, DE, RN, RT, CH, Id, MPSC>::new(sender_to_error.clone());
        let url = format!("ws://{}/ws", ADDRESS);
        web_socket.connect_to_url(&url).await;

        Self {
            _ph: PhantomData,
            my_wamp: web_socket,
        }
    }

    pub async fn sign_up(
        &self,
        check_from_cache_only: bool,
        input: &sign_up::Input,
    ) -> sign_up::Result {
        let (sender, receiver) = MPSC::channel();

        self.my_wamp
            .send_to_cache_actor(web_socket::Query {
                check_from_cache_only,
                sender: sender,
                data: input.clone().map_input(),
            })
            .await;

        let result = receiver.recv().await.unwrap();
        let result = sign_up::Input::unwrap(result);
        result
    }

    pub async fn sign_in(
        &self,
        check_from_cache_only: bool,
        input: &sign_in::Input,
    ) -> sign_in::Result {
        let (sender, receiver) = MPSC::channel();

        self.my_wamp
            .send_to_cache_actor(web_socket::Query {
                check_from_cache_only,
                sender: sender,
                data: input.clone().map_input(),
            })
            .await;

        let result = receiver.recv().await.unwrap();
        let result = sign_in::Input::unwrap(result);
        result
    }
}
