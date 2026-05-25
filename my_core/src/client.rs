use crate::{prelude::*, web_socket::AuthenticationOperations};

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

    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let (sender, receiver) = MPSC::channel();

        self.my_wamp
            .send_to_cache_actor(web_socket::Query::Authentication {
                sender: sender,
                data: input.clone().map_input(),
            })
            .await;

        let result = receiver.recv().await.unwrap();
        let result = sign_up::Input::unwrap(result);
        Ok(result)
    }

    pub async fn sign_in(&self, input: &sign_in::Input) -> Result<sign_in::Result, DynamicError> {
        todo!()
    }
}

impl AuthenticationOperations for sign_up::Input {
    type Ok = sign_up::Ok;
    type Err = sign_up::Error;

    async fn state_full_check<CH: CacheIO>(
        &self,
        state: &cache::State<CH>,
    ) -> Result<Self::Ok, Self::Err> {
        let mut err = Self::Err {
            new_uuid: None,
            user_id: None,
            name: None,
        };

        for (uuid, user) in &state.state_of_pending_txn.user {
            if user.user_id == self.user_id {
                err.user_id = Some(sign_up::UserIdError::Duplicated);
            }
            if uuid == &self.new_uuid {
                err.new_uuid = Some(RowIdError::Duplicated);
            }
        }

        if err != sign_up::Error::default() {
            return Err(err);
        }

        return Ok(sign_up::Ok { jwt: String::new() });
    }

    fn apply_change<CH: CacheIO>(&self, state: &mut cache::State<CH>) {
        state.state_of_pending_txn.user.insert(
            self.new_uuid.clone(),
            cache::tables::User {
                user_name: self.name.clone(),
                user_id: self.user_id.clone(),
                password: self.password.clone(),
            },
        );
    }

    fn map_input(self) -> push_data::AuthenticationMethodInput {
        push_data::AuthenticationMethodInput::SignUp(self)
    }

    fn map_result(result: Result<Self::Ok, Self::Err>) -> push_data::AuthenticationMethodResult {
        push_data::AuthenticationMethodResult::SignUp(result)
    }

    fn unwrap(result: push_data::AuthenticationMethodResult) -> Result<Self::Ok, Self::Err> {
        if let push_data::AuthenticationMethodResult::SignUp(result) = result {
            return result;
        }
        unreachable!()
    }
}
