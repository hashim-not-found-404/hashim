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
                        Err(_) => continue,
                    };

                    match msg {
                        WSMessage::Binary(received_data) => {
                            let recived_msg =
                                DE::decode::<web_socket::MessageType>(&received_data).unwrap();

                            match recived_msg {
                                web_socket::MessageType::TwoWay { id, path, payload } => {
                                    let payload = match path.as_str() {
                                        sign_up::PATH => {
                                            let input = DE::decode::<sign_up::Input>(&payload);

                                            let result = match input {
                                                Ok(input) => match state
                                                    .sign_up(&mut None, &input)
                                                    .await
                                                {
                                                    Ok(o) => Ok(o),
                                                    Err(_) => Err(HashimError::InternalServerError),
                                                },
                                                Err(_) => Err(HashimError::DecodingErrorAtServer),
                                            };

                                            DE::encode(&result)
                                        }
                                        sign_in::PATH => {
                                            let input = DE::decode::<sign_in::Input>(&payload);

                                            let result = match input {
                                                Ok(input) => {
                                                    let mut user_uuid = None;
                                                    let sign_in =
                                                        state.sign_in(&mut user_uuid, &input).await;

                                                    if let Some(user_uuid) = user_uuid {
                                                        let mut users = HashSet::new();
                                                        users.insert(user_uuid);

                                                        let subs = state
                                                            .get_table_of_subscribed_data(&users)
                                                            .await
                                                            .unwrap();

                                                        sender_to_broker.send(server_methods::MessageToBroker::Subscribe {
                                                            list_of_subscribtion: subs,
                                                            users_uuids:users,
                                                            sender_to_server: sender_to_server.clone(),
                                                        }).await.unwrap();
                                                    }

                                                    match sign_in {
                                                        Ok(o) => Ok(o),
                                                        Err(_) => {
                                                            Err(HashimError::InternalServerError)
                                                        }
                                                    }
                                                }
                                                Err(_) => Err(HashimError::DecodingErrorAtServer),
                                            };

                                            DE::encode(&result)
                                        }
                                        push_data::PATH => {
                                            let input = DE::decode::<push_data::Input>(&payload);

                                            let result = match input {
                                                Ok(input) => {
                                                    let mut resources =
                                                        HashSet::with_capacity(1000);
                                                    let mut users_uuids =
                                                        HashSet::with_capacity(10);

                                                    let push_data = state
                                                        .push_data(
                                                            &mut resources,
                                                            &mut users_uuids,
                                                            &input,
                                                        )
                                                        .await;

                                                    if !users_uuids.is_empty() {
                                                        let subs = state
                                                            .get_table_of_subscribed_data(
                                                                &users_uuids,
                                                            )
                                                            .await
                                                            .unwrap();

                                                        sender_to_broker.send(server_methods::MessageToBroker::Subscribe {
                                                            list_of_subscribtion: subs,
                                                            users_uuids,
                                                            sender_to_server: sender_to_server.clone(),
                                                        }).await.unwrap();
                                                    }

                                                    match push_data {
                                                        Ok(ok) => Ok(ok),
                                                        Err(_) => {
                                                            Err(HashimError::InternalServerError)
                                                        }
                                                    }
                                                }
                                                Err(_) => Err(HashimError::DecodingErrorAtServer),
                                            };

                                            DE::encode(&result)
                                        }
                                        _ => todo!(),
                                    };

                                    let msg_to_send =
                                        web_socket::MessageType::TwoWay { id, path, payload };
                                    let msg_to_send = DE::encode(&msg_to_send);
                                    match session.send_bin(msg_to_send).await {
                                        Ok(_) => continue,
                                        Err(_) => break,
                                    }
                                }
                                web_socket::MessageType::OneWay { path, payload } => todo!(),
                            };
                        }
                        WSMessage::Close => {
                            session.close().await.unwrap();
                            return;
                        }
                    }
                }
                Either::Two(a) => todo!(),
            }
        }
    });
}
