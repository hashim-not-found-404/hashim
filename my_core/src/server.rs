use crate::prelude::*;

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer {
    async fn send_bin(&self, bin: Vec<u8>) -> Result<(), DynamicError>;
    async fn receive(&self) -> Result<WSMessage, DynamicError>;
    async fn close(&self) -> Result<(), DynamicError>;
}

// TODO : make the server as actor
pub fn server_actor<DB, Cli, Jwt, Authentication, F, Id, DE, RT, WSS, MPSC>(
    state: server_methods::ServerMethods<DB, Cli, Jwt, Authentication, F, Id>,
    session: WSS,
    sender_to_broker: MPSC::Sender<server_methods::MessageToBroker<Id, MPSC>>,
) where
    DB: Database<Client = Cli> + 'static,
    Cli: DBClient<RowId = Id, HashedPassword = Authentication> + 'static,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id> + 'static,
    Authentication: HashedPassword + 'static,
    F: Functions + 'static,
    Id: RowId + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    WSS: WSServer + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    RT::spawn(async move {
        let (sender_to_server, receiver_to_server) =
            MPSC::channel::<Vec<server_methods::Resource>>();

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
                                                Ok(input) => match state.sign_up(&input).await {
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
                                                Ok(input) => match state.sign_in(&input).await {
                                                    Ok(o) => match o {
                                                        Ok((o, user_uuid)) => {
                                                            let subs = state
                                                                .get_table_of_subscribed_data(
                                                                    &user_uuid,
                                                                )
                                                                .await
                                                                .unwrap();

                                                            sender_to_broker.send(server_methods::MessageToBroker::Subscribe {
                                                                user_uuid: user_uuid,
                                                                list_of_subscribtion_for_company: subs.companies,
                                                                list_of_subscribtion_for_branch: subs.branches,
                                                                channel_to_send_to_facad: sender_to_server.clone()
                                                            }).await.unwrap();

                                                            Ok(Ok(o))
                                                        }
                                                        Err(e) => Ok(Err(e)),
                                                    },
                                                    Err(_) => Err(HashimError::InternalServerError),
                                                },
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
                        WSMessage::Close => todo!(),
                    }
                }
                Either::Two(a) => todo!(),
            }
        }
    });
}
