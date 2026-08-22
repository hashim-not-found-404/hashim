use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::request_response::messages::ResourcesDTO;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::DatabaseWrite;
use crate::accounting_domain::utility::types::RowId;
use crate::server::utility::server_traits;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::ListOfResources;
use crate::server::utility::server_traits::UserUuid;
use crate::utility::traits;
use crate::utility::traits::DynamicError;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::LogError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub trait DbBundle<Cli: DBClient>: 'static {
    type CreateAccount: for<'a> cases::create_account::DatabaseRead<Db<'a> = Cli::Txn<'a>, Error = DynamicError>;
    type WriteCreateAccount: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = cases::create_account::Ok>;

    type CreateAccountForBranch: for<'a> cases::create_account_for_branch::DatabaseRead<
            Db<'a> = Cli::Txn<'a>,
            Error = DynamicError,
        >;
    type WriteCreateAccountForBranch: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = cases::create_account_for_branch::Ok>;

    type CreateJournalEntry: for<'a> cases::create_journal_entry::DatabaseRead<
            Db<'a> = Cli::Txn<'a>,
            Error = DynamicError,
        >;
    type WriteCreateJournalEntry: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = cases::create_journal_entry::Ok>;

    type GetAllAccounts: for<'a> cases::get_all_accounts::DatabaseRead<Db<'a> = Cli, Error = DynamicError>;

    type GetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Cli, Error = DynamicError>;

    type CreateCompany: for<'a> cases::create_company::DatabaseRead<Db<'a> = Cli::Txn<'a>, Error = DynamicError>;
    type WriteCreateCompany: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = cases::create_company::Ok>;

    type CreateCompanyBranch: for<'a> cases::create_company_branch::DatabaseRead<
            Db<'a> = Cli::Txn<'a>,
            Error = DynamicError,
        >;
    type WriteCreateCompanyBranch: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = cases::create_company_branch::Ok>;

    type ListCompanyAndBranch: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Cli, Error = DynamicError>;

    type SignIn: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Cli, Error = DynamicError>;

    type SignUp: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Cli::Txn<'a>, Error = DynamicError>;
    type WriteSignUp: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = cases::sign_up::Ok>;
}

pub trait Database: 'static {
    type Client: DBClient;
    fn new() -> impl Future<Output = Self>;
    fn get_client(&self) -> impl Future<Output = Result<Self::Client, DynamicError>>;
}

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer: 'static {
    fn send_bin(&mut self, bin: Vec<u8>) -> impl Future<Output = Result<(), DynamicError>>;
    fn receive(&mut self) -> impl Future<Output = Result<WSMessage, DynamicError>>;
    fn close(self) -> impl Future<Output = Result<(), DynamicError>>;
}

pub struct ServerMethods<Mpsc: traits::MultiProducerSingleConsumer, Jwt: types::JWT, Db: Database> {
    database:                    Db,
    jwt:                         Jwt,
    pub(crate) sender_to_broker: Mpsc::Sender<MessageToBroker<Mpsc>>,
}

impl<
    Mpsc: traits::MultiProducerSingleConsumer,
    Jwt: types::JWT,
    Db: Database<Client = Cli>,
    Cli: DBClient,
> ServerMethods<Mpsc, Jwt, Db>
{
    pub async fn new<Rt: traits::Runtime>() -> Self {
        let (sender_to_broker, receiver_to_broker) = Mpsc::channel();
        Self::broker_actor::<Rt>(receiver_to_broker);

        Self {
            database: Db::new().await,
            jwt: Jwt::new(),
            sender_to_broker,
        }
    }

    pub fn server_actor<
        Rt: traits::Runtime,
        Ws: WSServer,
        Rn: traits::RandomNumber,
        Ed: traits::Coding,
        Id: types::RowId,
        Ti: traits::Time,
        Rg: traits::Regex,
        Auth: types::HashedPassword,
        Dbb: DbBundle<Cli>,
    >(
        self: Arc<Self>,
        mut session: Ws,
    ) {
        Rt::spawn_local(async move {
            let mut sender_to_broker = self.sender_to_broker.clone();
            let (sender_to_server, mut receiver_to_server) = Mpsc::channel::<Vec<ResourcesDTO>>();
            let connection_id = Rn::generate();

            loop {
                let result = Rt::select(session.receive(), receiver_to_server.recv()).await;
                match result {
                    traits::Either::One(msg) => {
                        let Ok(msg) = msg else {
                            break;
                        };

                        match msg {
                            WSMessage::Close => break,
                            WSMessage::Binary(received_data) => {
                                let Ok(input) = Ed::decode::<request_response::messages::FromClient>(
                                    &received_data,
                                ) else {
                                    if session
                                        .send_bin(Ed::encode(
                                            &request_response::messages::FromServer::Error(
                                                types::HashimError::InvalidDataFormat,
                                            ),
                                        ))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                    continue;
                                };

                                let Ok(mut client) = self.database.get_client().await else {
                                    if session
                                        .send_bin(Ed::encode(
                                            &request_response::messages::FromServer::Error(
                                                types::HashimError::InternalServerError,
                                            ),
                                        ))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                    continue;
                                };

                                dbg!(&input);
                                let mut side_effects = server_traits::SideEffects::default();
                                let output = push_data::<Id, Ti, Auth, Jwt, Cli, Dbb>(
                                    &input,
                                    &mut side_effects,
                                    &mut client,
                                    &self.jwt,
                                )
                                .await;

                                dbg!(&output);
                                match output {
                                    Ok(ok) => {
                                        if session
                                            .send_bin(Ed::encode(
                                                &request_response::messages::FromServer::PushData(
                                                    ok,
                                                ),
                                            ))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        if session
                                            .send_bin(Ed::encode(
                                                &request_response::messages::FromServer::Error(
                                                    types::HashimError::InternalServerError,
                                                ),
                                            ))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                }

                                if !side_effects.users_to_resubscribe.is_empty() {
                                    let Ok(subs) = get_table_of_subscribed_data::<Cli>(
                                        &mut client,
                                        &side_effects.users_to_resubscribe,
                                    )
                                    .await
                                    else {
                                        if session
                                            .send_bin(Ed::encode(
                                                &request_response::messages::FromServer::Error(
                                                    types::HashimError::InternalServerError,
                                                ),
                                            ))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                        continue;
                                    };

                                    if sender_to_broker
                                        .send(MessageToBroker::Subscribe {
                                            connection_id,
                                            list_of_subscribtion: subs,
                                            users_uuids: side_effects.users_to_resubscribe,
                                            sender_to_server: sender_to_server.clone(),
                                        })
                                        .await
                                        .log()
                                        .is_err()
                                    {
                                        break;
                                    }
                                }

                                if !side_effects.resource_to_broadcast_for_branch.is_empty()
                                    && sender_to_broker
                                        .send(MessageToBroker::Publish {
                                            connection_id,
                                            list_of_resources_for_branch: side_effects
                                                .resource_to_broadcast_for_branch,
                                        })
                                        .await
                                        .log()
                                        .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                    traits::Either::Two(wraped_resource) => {
                        let resource = wraped_resource.unwrap();
                        if session
                            .send_bin(Ed::encode(
                                &request_response::messages::FromServer::Resources(resource),
                            ))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }

            session.close().await.unwrap();

            sender_to_broker
                .send(MessageToBroker::Unsubscribe {
                    connection_id,
                })
                .await
                .unwrap();
        });
    }

    pub(crate) fn broker_actor<Rt: traits::Runtime>(
        mut receiver_to_broker: Mpsc::Receiver<MessageToBroker<Mpsc>>,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_pubsub_for_company: broker_functions::UserSubscribes =
                HashMap::with_capacity(1000);
            let mut pool_of_pubsub_for_branch: broker_functions::UserSubscribes =
                HashMap::with_capacity(10000);
            let mut pool_of_server_facad_channels: UserSenders<Mpsc> =
                HashMap::with_capacity(10000);

            loop {
                let message = receiver_to_broker.recv().await.unwrap();
                match message {
                    MessageToBroker::Subscribe {
                        connection_id,
                        list_of_subscribtion,
                        users_uuids,
                        sender_to_server,
                    } => {
                        for user_uuid in users_uuids {
                            pool_of_server_facad_channels
                                .entry(user_uuid)
                                .or_default()
                                .insert(connection_id, sender_to_server.clone());
                        }

                        broker_functions::merge_subscribes(
                            &mut pool_of_pubsub_for_branch,
                            list_of_subscribtion,
                        );
                    }
                    MessageToBroker::Unsubscribe {
                        connection_id,
                    } => {
                        let mut user_to_remove = Vec::new();

                        for (user_uuid, inner) in &mut pool_of_server_facad_channels {
                            if inner.remove(&connection_id).is_some() && inner.is_empty() {
                                user_to_remove.push(user_uuid.clone());
                            }
                        }

                        for user_uuid in user_to_remove {
                            pool_of_server_facad_channels.remove(&user_uuid);
                            broker_functions::unsubscribe(
                                &mut pool_of_pubsub_for_company,
                                &user_uuid,
                            );
                            broker_functions::unsubscribe(
                                &mut pool_of_pubsub_for_branch,
                                &user_uuid,
                            );
                        }
                    }
                    MessageToBroker::Publish {
                        connection_id,
                        list_of_resources_for_branch,
                    } => {
                        let mut resource_to_send = HashMap::new();

                        broker_functions::map_resource_to_subscribes(
                            &pool_of_pubsub_for_branch,
                            list_of_resources_for_branch,
                            &mut resource_to_send,
                        );

                        for (user_uuid, resource) in resource_to_send {
                            let channels = pool_of_server_facad_channels.get_mut(&user_uuid);

                            let Some(channels) = channels else {
                                dbg!("there is some problem here this should not happen");
                                continue;
                            };

                            for (connection_id1, mut sender) in channels.clone() {
                                if connection_id == connection_id1 {
                                    continue;
                                }
                                if sender.send(resource.clone()).await.is_err() {
                                    channels.remove(&connection_id1);
                                }
                            }

                            if channels.is_empty() {
                                pool_of_server_facad_channels.remove(&user_uuid);
                            }
                        }
                    }
                }
            }
        });
    }
}

async fn push_data<
    Id: types::RowId,
    Ti: traits::Time,
    Auth: types::HashedPassword,
    Jwt: types::JWT,
    Cli: DBClient,
    Dbb: DbBundle<Cli>,
>(
    input: &request_response::push_data::Input,
    side_effects: &mut server_traits::SideEffects,
    client: &mut Cli,
    jwt: &Jwt,
) -> Result<request_response::push_data::MyResult, DynamicError> {
    let mut the_return_result = request_response::push_data::MyResult {
        jwts:       Vec::with_capacity(input.jwts.len()),
        nonce:      Ok(()),
        operations: Vec::with_capacity(input.operations.len()),
    };

    let mut is_there_error = false;

    for jwt_value in &input.jwts {
        if let Some(user_uuid) = jwt.validate(jwt_value.clone()) {
            side_effects.authenticated_users.insert(user_uuid);
        } else {
            the_return_result.jwts.push(Err(types::JWTError::Invalid));
            is_there_error = true;
        }
    }

    if !Id::validate(&input.nonce) {
        the_return_result.nonce = Err(types::NonceError::Invalid);
        return Ok(the_return_result);
    }

    let is_nonce_used = client.write_nonce_if_not_used(&input.nonce).await?;

    if !check_nonce_if_valid::<Id>(&input.nonce, is_nonce_used) {
        the_return_result.nonce = Err(types::NonceError::Invalid);
    }

    if is_there_error {
        return Ok(the_return_result);
    }

    for transaction in &input.operations {
        let result = match &transaction.operation {
            request_response::push_data::OperationsInput::SignUp(input) => {
                request_response::push_data::OperationsResult::SignUp(
                    input
                        .handle_operation::<Id, Auth, Jwt, Cli, Dbb::SignUp, Dbb::WriteSignUp>(
                            side_effects,
                            client,
                            jwt,
                        )
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::SignIn(input) => {
                request_response::push_data::OperationsResult::SignIn(
                    input
                        .handle_operation::<Auth, Jwt, Cli, Dbb::SignIn>(side_effects, client, jwt)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateCompany(input) => {
                request_response::push_data::OperationsResult::CreateCompany(
                    input
                        .handle_operation::<Id, Cli, Dbb::CreateCompany, Dbb::WriteCreateCompany>(
                            side_effects,
                            client,
                        )
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(input) => {
                request_response::push_data::OperationsResult::CreateCompanyBranch(
                    input
                        .handle_operation::<Id, Cli, Dbb::CreateCompanyBranch,Dbb::WriteCreateCompanyBranch>(side_effects, client)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(input) => {
                request_response::push_data::OperationsResult::ListCompanyAndBranch(
                    input
                        .handle_operation::<Id, Cli, Dbb::ListCompanyAndBranch>(
                            side_effects,
                            client,
                        )
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateAccount(input) => {
                request_response::push_data::OperationsResult::CreateAccount(
                    input
                        .handle_operation::<Id, Cli, Dbb::CreateAccount,Dbb::WriteCreateAccount>(side_effects, client)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::GetAllAccounts(input) => {
                request_response::push_data::OperationsResult::GetAllAccounts(
                    input
                        .handle_operation::<Id, Cli, Dbb::GetAllAccounts>(side_effects, client)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateAccountForBranch(input) => {
                request_response::push_data::OperationsResult::CreateAccountForBranch(
                    input
                        .handle_operation::<Id, Cli, Dbb::CreateAccountForBranch,Dbb::WriteCreateAccountForBranch>(
                            side_effects,
                            client,
                        )
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::GetAllAccountsForBranch(input) => {
                request_response::push_data::OperationsResult::GetAllAccountsForBranch(
                    input
                        .handle_operation::<Id, Cli, Dbb::GetAllAccountsForBranch>(
                            side_effects,
                            client,
                        )
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateJournalEntry(input) => {
                request_response::push_data::OperationsResult::CreateJournalEntry(
                    input
                        .handle_operation::<Id, Ti, Cli, Dbb::CreateJournalEntry,Dbb::WriteCreateJournalEntry>(
                            side_effects,
                            client,
                        )
                        .await?,
                )
            }
        };

        the_return_result.operations.push(request_response::push_data::Txn {
            txn_number: transaction.txn_number,
            operation:  result,
        });
    }

    Ok(the_return_result)
}

fn check_nonce_if_valid<Id: RowId>(nonce: &types::UuidType, is_used: bool) -> bool {
    if is_used {
        return false;
    }

    let Some(nonce) = Id::get_time_as_seconds(nonce) else {
        return false;
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let max_future = 5;

    if nonce > now + max_future {
        return false;
    }

    let max_age = 300;

    if now.saturating_sub(nonce) > max_age {
        return false;
    }

    true
}

async fn get_table_of_subscribed_data<Cli: DBClient>(
    client: &mut Cli,
    users_uuids: &HashSet<UserUuid>,
) -> Result<broker_functions::UserSubscribes, DynamicError> {
    let roles = client.read_roles_for_user(users_uuids).await?;

    let mut subs: broker_functions::UserSubscribes = HashMap::new();

    for (company, users_roles) in roles.companies {
        // let mut users_subscribes = HashMap::new();

        // for (user, roles) in users_roles {
        //     users_subscribes.insert(user);
        // }

        // subs.companies.insert(company, users_subscribes);
    }

    for (branch, users_roles) in roles.branches {
        // let mut users_subscribes = HashMap::new();

        // for (user, roles) in users_roles {
        //     let subscribes = role_to_subscribe_mapping(roles);
        //     users_subscribes.insert(user, subscribes);
        // }

        // subs.companies.insert(branch, users_subscribes);
    }

    Ok(subs)
}

mod broker_functions {
    use crate::accounting_domain::request_response::messages::ResourcesDTO;
    use crate::server::utility::server_traits::BranchUuid;
    use crate::server::utility::server_traits::ListOfResources;
    use crate::server::utility::server_traits::UserUuid;
    use std::collections::HashMap;
    use std::collections::HashSet;

    pub(crate) type UserSubscribes = HashMap<BranchUuid, HashSet<UserUuid>>;

    pub(crate) fn map_resource_to_subscribes(
        pool_of_pubsub: &UserSubscribes,
        list_of_resources: ListOfResources,
        resource_to_send: &mut HashMap<UserUuid, Vec<ResourcesDTO>>,
    ) {
        for (branch, resources) in list_of_resources {
            let user_and_subscribe = pool_of_pubsub.get(&branch);
            let Some(user_and_subscribe) = user_and_subscribe else {
                dbg!("there is some problem here this should not happen");
                continue;
            };

            for user_uuid in user_and_subscribe {
                for resource in resources.clone() {
                    resource_to_send.entry(user_uuid.clone()).or_default().push(resource.clone());
                }
            }
        }
    }

    pub(crate) fn unsubscribe(pool_of_pubsub: &mut UserSubscribes, user_uuid: &UserUuid) {
        pool_of_pubsub.retain(|_, users_and_subs| {
            users_and_subs.remove(user_uuid);
            !users_and_subs.is_empty()
        });
    }

    pub(crate) fn merge_subscribes(
        pool_of_pubsub: &mut UserSubscribes,
        list_of_subscribtion: UserSubscribes,
    ) {
        for (branch, users_subscribes) in list_of_subscribtion {
            for user_uuid in users_subscribes {
                pool_of_pubsub.entry(branch.clone()).or_default().insert(user_uuid);
            }
        }
    }
}

type UserSenders<Mpsc> = HashMap<
    UserUuid,
    HashMap<u64, <Mpsc as traits::MultiProducerSingleConsumer>::Sender<Vec<ResourcesDTO>>>, // because user may have multiple web socket connection
>;

pub(crate) enum MessageToBroker<Mpsc: traits::MultiProducerSingleConsumer> {
    Subscribe {
        connection_id:        u64,
        list_of_subscribtion: broker_functions::UserSubscribes,
        users_uuids:          HashSet<UserUuid>,
        sender_to_server:     Mpsc::Sender<Vec<ResourcesDTO>>,
    },
    Unsubscribe {
        connection_id: u64,
    },
    Publish {
        connection_id:                u64,
        list_of_resources_for_branch: ListOfResources,
    },
}
