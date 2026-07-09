use crate::{
    accounting_domain::{
        cases::{self, RowId},
        request_response, types,
    },
    server::{server_traits::DBClient, server_types, use_cases::ServerOperations},
    utility::{
        traits::{self, Receiver, Sender},
        utils::{self, HashMapWithHashMapValue, HashMapWithVectorValue},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub trait Database: 'static {
    type Client: DBClient;
    fn new() -> impl Future<Output = Self>;
    fn get_client(&self) -> impl Future<Output = Result<Self::Client, utils::DynamicError>>;
}

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer: 'static {
    fn send_bin(&mut self, bin: Vec<u8>) -> impl Future<Output = Result<(), utils::DynamicError>>;
    fn receive(&mut self) -> impl Future<Output = Result<WSMessage, utils::DynamicError>>;
    fn close(self) -> impl Future<Output = Result<(), utils::DynamicError>>;
}

pub struct ServerMethods<Mpsc: traits::MultiProducerSingleConsumer, Jwt: cases::JWT, Db: Database> {
    database: Db,
    jwt: Jwt,
    pub(crate) sender_to_broker:
        <Mpsc as traits::MultiProducerSingleConsumer>::Sender<MessageToBroker<Mpsc>>,
}

impl<
    Mpsc: traits::MultiProducerSingleConsumer,
    Jwt: cases::JWT,
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
        Id: cases::RowId,
        Rg: traits::Regex,
        Auth: cases::HashedPassword,
    >(
        self: Arc<Self>,
        mut session: Ws,
    ) {
        Rt::spawn_local(async move {
            let mut sender_to_broker = self.sender_to_broker.clone();
            let (sender_to_server, mut receiver_to_server) =
                Mpsc::channel::<Vec<types::ResourceInfo>>();
            let connection_id = Rn::generate();

            loop {
                let result = Rt::select(session.receive(), receiver_to_server.recv()).await;
                match result {
                    traits::Either::One(msg) => {
                        let msg = match msg {
                            Ok(msg) => msg,
                            Err(_) => break,
                        };

                        match msg {
                            WSMessage::Close => break,
                            WSMessage::Binary(received_data) => {
                                let input = match Ed::decode::<request_response::messages::FromClient>(
                                    &received_data,
                                ) {
                                    Ok(ok) => ok,
                                    Err(_) => {
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
                                    }
                                };

                                let mut client = match self.database.get_client().await {
                                    Ok(ok) => ok,
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
                                        continue;
                                    }
                                };

                                dbg!(&input);
                                let mut side_effects = server_types::SideEffects::default();
                                let output = push_data::<Rn, Id, Rg, Auth, Jwt, Cli>(
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
                                    let subs = match get_table_of_subscribed_data::<Cli>(
                                        &mut client,
                                        &side_effects.users_to_resubscribe,
                                    )
                                    .await
                                    {
                                        Ok(ok) => ok,
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
                                            continue;
                                        }
                                    };

                                    sender_to_broker
                                        .send(MessageToBroker::Subscribe {
                                            connection_id,
                                            list_of_subscribtion: subs,
                                            users_uuids: side_effects.users_to_resubscribe,
                                            sender_to_server: sender_to_server.clone(),
                                        })
                                        .await
                                        .unwrap();
                                }

                                if !side_effects.resource_to_broadcast_for_company.is_empty()
                                    || !side_effects.resource_to_broadcast_for_branch.is_empty()
                                {
                                    sender_to_broker
                                        .send(MessageToBroker::Publish {
                                            connection_id,
                                            list_of_resources_for_company: side_effects
                                                .resource_to_broadcast_for_company,
                                            list_of_resources_for_branch: side_effects
                                                .resource_to_broadcast_for_branch,
                                        })
                                        .await
                                        .unwrap();
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
                .send(MessageToBroker::Unsubscribe { connection_id })
                .await
                .unwrap();
        });
    }

    pub(crate) fn broker_actor<Rt: traits::Runtime>(
        mut receiver_to_broker: <Mpsc as traits::MultiProducerSingleConsumer>::Receiver<
            MessageToBroker<Mpsc>,
        >,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_pubsub_for_company: UserSubscribes = HashMap::with_capacity(1000);
            let mut pool_of_pubsub_for_branch: UserSubscribes = HashMap::with_capacity(10000);
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
                            pool_of_server_facad_channels.nested_insert(
                                user_uuid,
                                connection_id,
                                sender_to_server.clone(),
                            );
                        }

                        broker_functions::merge_subscribes(
                            &mut pool_of_pubsub_for_company,
                            list_of_subscribtion.companies,
                        );

                        broker_functions::merge_subscribes(
                            &mut pool_of_pubsub_for_branch,
                            list_of_subscribtion.branches,
                        );
                    }
                    MessageToBroker::Unsubscribe { connection_id } => {
                        let mut user_to_remove = Vec::new();

                        for (user_uuid, inner) in pool_of_server_facad_channels.iter_mut() {
                            if inner.remove(&connection_id).is_some() {
                                if inner.is_empty() {
                                    user_to_remove.push(user_uuid.clone());
                                }
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
                        list_of_resources_for_company,
                        list_of_resources_for_branch,
                    } => {
                        let mut resource_to_send: server_types::ListOfResources = HashMap::new();

                        broker_functions::map_resource_to_subscribes(
                            &pool_of_pubsub_for_company,
                            list_of_resources_for_company,
                            &mut resource_to_send,
                        );

                        broker_functions::map_resource_to_subscribes(
                            &pool_of_pubsub_for_branch,
                            list_of_resources_for_branch,
                            &mut resource_to_send,
                        );

                        for (user_uuid, resource) in resource_to_send {
                            let channels = pool_of_server_facad_channels.get_mut(&user_uuid);

                            match channels {
                                Some(channels) => {
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
                                None => {
                                    dbg!("there is some problem here this should not happen");
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

async fn push_data<
    Rn: traits::RandomNumber,
    Id: cases::RowId,
    Rg: traits::Regex,
    Auth: cases::HashedPassword,
    Jwt: cases::JWT,
    Cli: DBClient,
>(
    input: &request_response::push_data::Input,
    side_effects: &mut server_types::SideEffects,
    client: &mut Cli,
    jwt: &Jwt,
) -> Result<request_response::push_data::MyResult, utils::DynamicError> {
    let mut the_return_result = request_response::push_data::MyResult {
        jwts: Vec::with_capacity(input.jwts.len()),
        nonce: Ok(()),
        operations: Vec::with_capacity(input.operations.len()),
    };

    let mut is_there_error = false;

    for jwt_value in &input.jwts {
        match jwt.validate(jwt_value.clone()) {
            Some(user_uuid) => {
                side_effects.authenticated_users.insert(user_uuid);
            }
            None => {
                the_return_result.jwts.push(Err(types::JWTError::Invalid));

                is_there_error = true;
            }
        }
    }

    if !Id::validate(&input.nonce) {
        the_return_result.nonce = Err(types::NonceError::Invalid);
        return Ok(the_return_result);
    };

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
                        .handle_operation::<Rn, Id, Rg, Auth, Jwt, Cli>(side_effects, client, &jwt)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::SignIn(input) => {
                request_response::push_data::OperationsResult::SignIn(
                    input
                        .handle_operation::<Rn, Id, Rg, Auth, Jwt, Cli>(side_effects, client, &jwt)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateCompany(input) => {
                request_response::push_data::OperationsResult::CreateCompany(
                    input
                        .handle_operation::<Rn, Id, Rg, Auth, Jwt, Cli>(side_effects, client, &jwt)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::CreateCompanyBranch(input) => {
                request_response::push_data::OperationsResult::CreateCompanyBranch(
                    input
                        .handle_operation::<Rn, Id, Rg, Auth, Jwt, Cli>(side_effects, client, &jwt)
                        .await?,
                )
            }
            request_response::push_data::OperationsInput::ListCompanyAndBranch(input) => {
                request_response::push_data::OperationsResult::ListCompanyAndBranch(
                    input
                        .handle_operation::<Rn, Id, Rg, Auth, Jwt, Cli>(side_effects, client, &jwt)
                        .await?,
                )
            }
        };

        the_return_result
            .operations
            .push(request_response::push_data::Txn {
                txn_number: transaction.txn_number,
                operation: result,
            });
    }

    return Ok(the_return_result);
}

fn check_nonce_if_valid<Id: RowId>(nonce: &types::UuidType, is_used: bool) -> bool {
    if is_used {
        return false;
    }

    let nonce = match Id::get_time_as_seconds(nonce) {
        Some(nonce) => nonce,
        None => return false,
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u64;

    // 1. Reject future timestamps (more than 5 seconds ahead)
    let max_future = 5; // 5 seconds tolerance for clock drift

    if nonce > now + max_future {
        return false; // Future nonce → reject immediately
    }

    // 2. Reject old timestamps (more than 5 minutes old)
    let max_age = 300; // 5 minutes

    if now.saturating_sub(nonce) > max_age {
        return false; // Too old
    }

    true
}

async fn get_table_of_subscribed_data<Cli: DBClient>(
    client: &mut Cli,
    users_uuids: &HashSet<types::UuidType>,
) -> Result<AllSubscribes, utils::DynamicError> {
    let roles = client.read_roles_for_user(users_uuids).await?;

    let mut subs = AllSubscribes {
        companies: HashMap::new(),
        branches: HashMap::new(),
    };

    for (company, users_roles) in roles.companies {
        let mut users_subscribes = HashMap::new();

        for (user, roles) in users_roles {
            let subscribes = role_to_subscribe_mapping(roles);
            users_subscribes.insert(user, subscribes);
        }

        subs.companies.insert(company, users_subscribes);
    }

    for (branch, users_roles) in roles.branches {
        let mut users_subscribes = HashMap::new();

        for (user, roles) in users_roles {
            let subscribes = role_to_subscribe_mapping(roles);
            users_subscribes.insert(user, subscribes);
        }

        subs.companies.insert(branch, users_subscribes);
    }

    Ok(subs)
}

mod broker_functions {
    use super::*;

    pub(crate) fn map_resource_to_subscribes(
        pool_of_pubsub: &UserSubscribes,
        list_of_resources: server_types::ListOfResources,
        resource_to_send: &mut server_types::ListOfResources,
    ) {
        for (company, resource) in list_of_resources {
            let user_and_subscribe = pool_of_pubsub.get(&company);
            match user_and_subscribe {
                Some(user_and_subscribe) => {
                    for (user_uuid, subscribe) in user_and_subscribe {
                        let resource_for_user =
                            resource_filtering_based_on_subscribe(subscribe, &resource);

                        resource_to_send.insert_append(user_uuid.clone(), resource_for_user);
                    }
                }
                None => {
                    dbg!("there is some problem here this should not happen");
                    continue;
                }
            }
        }
    }

    pub(crate) fn unsubscribe(pool_of_pubsub: &mut UserSubscribes, user_uuid: &types::UuidType) {
        pool_of_pubsub.retain(|_, users_and_subs| {
            users_and_subs.remove(user_uuid);
            !users_and_subs.is_empty()
        });
    }

    pub(crate) fn merge_subscribes(
        pool_of_pubsub: &mut UserSubscribes,
        list_of_subscribtion: UserSubscribes,
    ) {
        for (company, users_subscribes) in list_of_subscribtion {
            for (user_uuid, subscribes) in users_subscribes {
                pool_of_pubsub.nested_insert(company.clone(), user_uuid, subscribes);
            }
        }
    }

    fn resource_filtering_based_on_subscribe(
        subscribe: &HashSet<types::Subscribe>,
        resource: &Vec<types::ResourceInfo>,
    ) -> Vec<types::ResourceInfo> {
        let mut new_resource = Vec::new();

        for one_resource in resource {
            match one_resource.resource {
                types::Resource::Jwt(_) => {}
                types::Resource::TableUserFieldName(_) => {
                    if subscribe.contains(&types::Subscribe::TableUserFieldName) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableUserFieldId(_) => {
                    if subscribe.contains(&types::Subscribe::TableUserFieldId) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableCompanyFieldName(_) => {
                    if subscribe.contains(&types::Subscribe::TableCompanyFieldName) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableCompanyBranchFieldName(_) => {
                    if subscribe.contains(&types::Subscribe::TableCompanyBranchFieldName) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableCompanyBranchFieldCompanyBelong(_) => {
                    if subscribe.contains(&types::Subscribe::TableCompanyBranchFieldCompanyBelong) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableCompanyBranchFieldCurrency(_) => {
                    if subscribe.contains(&types::Subscribe::TableCompanyBranchFieldCurrency) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableCompanyBranchFieldLocation(_) => {
                    if subscribe.contains(&types::Subscribe::TableCompanyBranchFieldLocation) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableCompanyFieldCurrency(_) => {
                    if subscribe.contains(&types::Subscribe::TableCompanyFieldCurrency) {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableAccessControlForCompanyFieldRole(_) => {
                    if subscribe.contains(&types::Subscribe::TableAccessControlForCompanyFieldRole)
                    {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableAccessControlForCompanyFieldUser(_) => {
                    if subscribe.contains(&types::Subscribe::TableAccessControlForCompanyFieldUser)
                    {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableAccessControlForCompanyFieldDataGroup(_) => {
                    if subscribe
                        .contains(&types::Subscribe::TableAccessControlForCompanyFieldDataGroup)
                    {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableAccessControlForCompanyBranchFieldRole(_) => {
                    if subscribe
                        .contains(&types::Subscribe::TableAccessControlForCompanyBranchFieldRole)
                    {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableAccessControlForCompanyBranchFieldUser(_) => {
                    if subscribe
                        .contains(&types::Subscribe::TableAccessControlForCompanyBranchFieldUser)
                    {
                        new_resource.push(one_resource.clone());
                    }
                }
                types::Resource::TableAccessControlForCompanyBranchFieldDataGroup(_) => {
                    if subscribe.contains(
                        &types::Subscribe::TableAccessControlForCompanyBranchFieldDataGroup,
                    ) {
                        new_resource.push(one_resource.clone());
                    }
                }
            }
        }

        new_resource
    }
}

fn role_to_subscribe_mapping(roles: Vec<types::Role>) -> HashSet<types::Subscribe> {
    let mut subscribes = HashSet::with_capacity(200);

    for role in roles {
        match role {
            types::Role::Manager => {
                subscribes.insert(types::Subscribe::TableUserFieldName);
                subscribes.insert(types::Subscribe::TableUserFieldId);
                subscribes.insert(types::Subscribe::TableCompanyFieldName);
                subscribes.insert(types::Subscribe::TableCompanyFieldCurrency);
                subscribes.insert(types::Subscribe::TableCompanyBranchFieldName);
                subscribes.insert(types::Subscribe::TableCompanyBranchFieldCompanyBelong);
                subscribes.insert(types::Subscribe::TableAccessControlForCompanyFieldRole);
                subscribes.insert(types::Subscribe::TableAccessControlForCompanyFieldUser);
                subscribes.insert(types::Subscribe::TableAccessControlForCompanyFieldDataGroup);
            }
            types::Role::CoManager => {
                subscribes.insert(types::Subscribe::TableUserFieldName);
                subscribes.insert(types::Subscribe::TableUserFieldId);
                subscribes.insert(types::Subscribe::TableCompanyFieldName);
                subscribes.insert(types::Subscribe::TableCompanyFieldCurrency);
                subscribes.insert(types::Subscribe::TableCompanyBranchFieldName);
                subscribes.insert(types::Subscribe::TableCompanyBranchFieldCompanyBelong);
                subscribes.insert(types::Subscribe::TableAccessControlForCompanyFieldRole);
                subscribes.insert(types::Subscribe::TableAccessControlForCompanyFieldUser);
                subscribes.insert(types::Subscribe::TableAccessControlForCompanyFieldDataGroup);
            }
        }
    }

    subscribes.shrink_to_fit();
    subscribes
}

pub(crate) struct AllSubscribes {
    pub(crate) companies: UserSubscribes,
    pub(crate) branches: UserSubscribes,
}

type UserSubscribes = HashMap<
    types::UuidType, // company uuid or branch
    HashMap<
        types::UuidType, // user uuid
        HashSet<types::Subscribe>,
    >,
>;

type UserSenders<Mpsc: traits::MultiProducerSingleConsumer> = HashMap<
    types::UuidType,                                      // user uuid
    HashMap<u64, Mpsc::Sender<Vec<types::ResourceInfo>>>, // because user may have multiple web socket connection
>;

pub(crate) enum MessageToBroker<Mpsc: traits::MultiProducerSingleConsumer> {
    Subscribe {
        connection_id: u64,
        list_of_subscribtion: AllSubscribes,
        users_uuids: HashSet<types::UuidType>,
        sender_to_server: Mpsc::Sender<Vec<types::ResourceInfo>>,
    },
    Unsubscribe {
        connection_id: u64,
    },
    Publish {
        connection_id: u64,
        list_of_resources_for_company: server_types::ListOfResources,
        list_of_resources_for_branch: server_types::ListOfResources,
    },
}
