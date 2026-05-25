use crate::prelude::*;

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer {
    async fn send_bin(&mut self, bin: Vec<u8>) -> Result<(), DynamicError>;
    async fn receive(&mut self) -> Result<WSMessage, DynamicError>;
    async fn close(self) -> Result<(), DynamicError>;
}

pub fn server_actor<DB, Cli, Jwt, Authentication, F, Id, DE, RT, WSS, MPSC>(
    state: Arc<server_methods::ServerMethods<DB, Cli, Jwt, Authentication, F, Id, MPSC, RT>>,
    mut session: WSS,
    sender_to_broker: MPSC::Sender<server_methods::MessageToBroker<Id, MPSC>>,
) where
    DB: Database<Client = Cli> + 'static,
    Cli: DBClient<RowId = Id, HashedPassword = Authentication> + 'static,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id, JsonWebToken = String> + 'static,
    Authentication: HashedPassword + 'static,
    F: Functions + 'static,
    Id: RowId + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    WSS: WSServer + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    RT::spawn_local(async move {
        let (sender_to_server, receiver_to_server) = MPSC::channel::<Vec<ResourceInfo>>();

        loop {
            let result = RT::select(session.receive(), receiver_to_server.recv()).await;
            match result {
                Either::One(msg) => {
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(_) => break,
                    };

                    match msg {
                        WSMessage::Binary(received_data) => {
                            let input = DE::decode::<messages::FromClient>(&received_data);

                            let mut resources = HashSet::with_capacity(1000);
                            let mut users_uuids = HashSet::with_capacity(10);
                            // TODO : get db client here

                            let result = match input {
                                Ok(input) => state
                                    .push_data(&mut resources, &mut users_uuids, &input)
                                    .await
                                    .map_err(|e| {
                                        dbg!(e);
                                        HashimError::InternalServerError
                                    }),
                                Err(_) => Err(HashimError::InvalidDataFormat),
                            };

                            if let Err(_) = session
                                .send_bin(DE::encode(&messages::FromServer::PushData(result)))
                                .await
                            {
                                break;
                            }

                            if !users_uuids.is_empty() {
                                let subs = state
                                    .get_table_of_subscribed_data(&users_uuids)
                                    .await
                                    .unwrap();

                                sender_to_broker
                                    .send(server_methods::MessageToBroker::Subscribe {
                                        list_of_subscribtion: subs,
                                        users_uuids,
                                        sender_to_server: sender_to_server.clone(),
                                    })
                                    .await
                                    .unwrap();
                            }
                        }
                        WSMessage::Close => break,
                    }
                }
                Either::Two(a) => todo!(),
            }
        }

        session.close().await.unwrap();
    });
}
