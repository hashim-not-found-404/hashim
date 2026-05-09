use crate::prelude::*;

pub async fn server<DB, Cli, Jwt, Authentication, F, Id, DE>(
    received_data: &Vec<u8>,
    state: &server_methods::ServerMethods<DB, Cli, Jwt, Authentication, F, Id>,
) -> Vec<u8>
where
    DB: Database<Client = Cli>,
    Cli: DBClient<RowId = Id, HashedPassword = Authentication>,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId,
    DE: Coding,
{
    let recived_msg = DE::decode::<web_socket::MessageType>(received_data).unwrap();

    let msg_to_send = match recived_msg {
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
                            Ok(o) => Ok(o),
                            Err(_) => Err(HashimError::InternalServerError),
                        },
                        Err(_) => Err(HashimError::DecodingErrorAtServer),
                    };

                    DE::encode(&result)
                }
                _ => todo!(),
            };

            let msg_to_send = web_socket::MessageType::TwoWay { id, path, payload };

            msg_to_send
        }
        web_socket::MessageType::OneWay { path, payload } => todo!(),
    };

    DE::encode(&msg_to_send)
}
