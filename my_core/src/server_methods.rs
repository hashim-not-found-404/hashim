use crate::prelude::*;

fn check_nonce_if_valid<Id: RowId>(nonce: &Id, is_used: bool) -> bool {
    if is_used {
        return false;
    }

    let nonce = nonce.get_time_as_seconds();

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

pub struct ServerMethods<At: AllServerTypes> {
    database: At::Db,
    jwt: At::Jwt,
    pub sender_to_broker:
        <At::Mpsc as MultiProducerSingleConsumer>::Sender<MessageToBroker<At::Id, At::Mpsc>>,
}

impl<At: AllServerTypes> ServerMethods<At> {
    pub async fn new() -> Self {
        let (sender_to_broker, receiver_to_broker) = At::Mpsc::channel();
        Self::broker_actor(receiver_to_broker);

        Self {
            database: At::Db::new().await,
            jwt: At::Jwt::new(),
            sender_to_broker,
        }
    }

    pub fn server_actor(self: Arc<Self>, mut session: At::Ws) {
        At::Rt::spawn_local(async move {
            let mut sender_to_broker = self.sender_to_broker.clone();
            let (sender_to_server, mut receiver_to_server) =
                At::Mpsc::channel::<Vec<ResourceInfo>>();
            let connection_id = At::Rn::generate();

            loop {
                let result = At::Rt::select(session.receive(), receiver_to_server.recv()).await;
                match result {
                    Either::One(msg) => {
                        let msg = match msg {
                            Ok(msg) => msg,
                            Err(_) => break,
                        };

                        match msg {
                            WSMessage::Close => break,
                            WSMessage::Binary(received_data) => {
                                let input =
                                    match At::De::decode::<messages::FromClient>(&received_data) {
                                        Ok(ok) => ok,
                                        Err(_) => {
                                            if session
                                                .send_bin(At::De::encode(
                                                    &messages::FromServer::Error(
                                                        HashimError::InvalidDataFormat,
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
                                            .send_bin(At::De::encode(&messages::FromServer::Error(
                                                HashimError::InternalServerError,
                                            )))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                        continue;
                                    }
                                };

                                dbg!(&input);
                                let mut side_effects = SideEffects::<At::Id>::default();
                                let output = push_data::<At>(
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
                                            .send_bin(At::De::encode(
                                                &messages::FromServer::PushData(ok),
                                            ))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        if session
                                            .send_bin(At::De::encode(&messages::FromServer::Error(
                                                HashimError::InternalServerError,
                                            )))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                }

                                if !side_effects.users_to_resubscribe.is_empty() {
                                    let subs = match get_table_of_subscribed_data::<At>(
                                        &mut client,
                                        &side_effects.users_to_resubscribe,
                                    )
                                    .await
                                    {
                                        Ok(ok) => ok,
                                        Err(_) => {
                                            if session
                                                .send_bin(At::De::encode(
                                                    &messages::FromServer::Error(
                                                        HashimError::InternalServerError,
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
                                        .send(server_methods::MessageToBroker::Subscribe {
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
                    Either::Two(wraped_resource) => {
                        let resource = wraped_resource.unwrap();
                        if session
                            .send_bin(At::De::encode(&messages::FromServer::Resources(resource)))
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

    pub fn broker_actor(
        mut receiver_to_broker: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<
            MessageToBroker<At::Id, At::Mpsc>,
        >,
    ) {
        At::Rt::spawn_local(async move {
            let mut pool_of_pubsub_for_company: UserSubscribes<At::Id> =
                HashMap::with_capacity(1000);
            let mut pool_of_pubsub_for_branch: UserSubscribes<At::Id> =
                HashMap::with_capacity(10000);
            let mut pool_of_server_facad_channels: UserSenders<At::Id, At::Mpsc> =
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
                        let mut resource_to_send: ListOfResources<At::Id /* user id */> =
                            HashMap::new();

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

async fn push_data<At: AllServerTypes>(
    input: &push_data::Input,
    side_effects: &mut server_methods::SideEffects<At::Id>,
    client: &mut At::Cli,
    jwt: &At::Jwt,
) -> Result<push_data::Result, DynamicError> {
    let mut the_return_result = push_data::Result {
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
                the_return_result.jwts.push(Err(JWTError::Invalid));

                is_there_error = true;
            }
        }
    }

    let nonce = match At::Id::try_from(&input.nonce) {
        Ok(nonce) => nonce,
        Err(_) => {
            the_return_result.nonce = Err(NonceError::Invalid);
            return Ok(the_return_result);
        }
    };

    let is_nonce_used = client.write_nonce_if_not_used(&nonce).await?;

    if !check_nonce_if_valid::<At::Id>(&nonce, is_nonce_used) {
        the_return_result.nonce = Err(NonceError::Invalid);
    }

    if is_there_error {
        return Ok(the_return_result);
    }

    for transaction in &input.operations {
        let result = match &transaction.operation {
            push_data::OperationsInput::SignUp(input) => push_data::OperationsResult::SignUp(
                input
                    .handle_operation::<At>(side_effects, client, &jwt)
                    .await?,
            ),
            push_data::OperationsInput::SignIn(input) => push_data::OperationsResult::SignIn(
                input
                    .handle_operation::<At>(side_effects, client, &jwt)
                    .await?,
            ),
            push_data::OperationsInput::CreateCompany(input) => {
                push_data::OperationsResult::CreateCompany(
                    input
                        .handle_operation::<At>(side_effects, client, &jwt)
                        .await?,
                )
            }
            push_data::OperationsInput::CreateCompanyBranch(input) => {
                push_data::OperationsResult::CreateCompanyBranch(
                    input
                        .handle_operation::<At>(side_effects, client, &jwt)
                        .await?,
                )
            }
            push_data::OperationsInput::ListCompanyAndBranch(input) => {
                push_data::OperationsResult::ListCompanyAndBranch(
                    input
                        .handle_operation::<At>(side_effects, client, &jwt)
                        .await?,
                )
            }
        };

        the_return_result.operations.push(push_data::Txn {
            txn_number: transaction.txn_number,
            operation: result,
        });
    }

    return Ok(the_return_result);
}

async fn get_table_of_subscribed_data<At: AllServerTypes>(
    client: &mut At::Cli,
    users_uuids: &HashSet<At::Id>,
) -> Result<AllSubscribes<At::Id>, DynamicError> {
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

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer {
    fn send_bin(&mut self, bin: Vec<u8>) -> impl Future<Output = Result<(), DynamicError>>;
    fn receive(&mut self) -> impl Future<Output = Result<WSMessage, DynamicError>>;
    fn close(self) -> impl Future<Output = Result<(), DynamicError>>;
}
mod broker_functions {
    use super::*;

    pub fn map_resource_to_subscribes<Id: RowId>(
        pool_of_pubsub: &UserSubscribes<Id>,
        list_of_resources: ListOfResources<Id>,
        resource_to_send: &mut ListOfResources<Id>,
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

    pub fn unsubscribe<Id: RowId>(pool_of_pubsub: &mut UserSubscribes<Id>, user_uuid: &Id) {
        pool_of_pubsub.retain(|_, users_and_subs| {
            users_and_subs.remove(user_uuid);
            !users_and_subs.is_empty()
        });
    }

    pub fn merge_subscribes<Id: RowId>(
        pool_of_pubsub: &mut UserSubscribes<Id>,
        list_of_subscribtion: UserSubscribes<Id>,
    ) {
        for (company, users_subscribes) in list_of_subscribtion {
            for (user_uuid, subscribes) in users_subscribes {
                pool_of_pubsub.nested_insert(company.clone(), user_uuid, subscribes);
            }
        }
    }

    fn resource_filtering_based_on_subscribe(
        subscribe: &HashSet<Subscribe>,
        resource: &Vec<ResourceInfo>,
    ) -> Vec<ResourceInfo> {
        let mut new_resource = Vec::new();

        for one_resource in resource {
            match one_resource.resource {
                Resource::Jwt(_) => {}
                Resource::TableUserFieldName(_) => {
                    if subscribe.contains(&Subscribe::TableUserFieldName) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableUserFieldId(_) => {
                    if subscribe.contains(&Subscribe::TableUserFieldId) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableCompanyFieldName(_) => {
                    if subscribe.contains(&Subscribe::TableCompanyFieldName) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableCompanyBranchFieldName(_) => {
                    if subscribe.contains(&Subscribe::TableCompanyBranchFieldName) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableCompanyBranchFieldCompanyBelong(_) => {
                    if subscribe.contains(&Subscribe::TableCompanyBranchFieldCompanyBelong) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableCompanyFieldCurrency(_) => {
                    if subscribe.contains(&Subscribe::TableCompanyFieldCurrency) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableAccessControlForCompanyFieldRole(_) => {
                    if subscribe.contains(&Subscribe::TableAccessControlForCompanyFieldRole) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableAccessControlForCompanyFieldUser(_) => {
                    if subscribe.contains(&Subscribe::TableAccessControlForCompanyFieldUser) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableAccessControlForCompanyFieldDataGroup(_) => {
                    if subscribe.contains(&Subscribe::TableAccessControlForCompanyFieldDataGroup) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableAccessControlForCompanyBranchFieldRole(_) => {
                    if subscribe.contains(&Subscribe::TableAccessControlForCompanyBranchFieldRole) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableAccessControlForCompanyBranchFieldUser(_) => {
                    if subscribe.contains(&Subscribe::TableAccessControlForCompanyBranchFieldUser) {
                        new_resource.push(one_resource.clone());
                    }
                }
                Resource::TableAccessControlForCompanyBranchFieldDataGroup(_) => {
                    if subscribe
                        .contains(&Subscribe::TableAccessControlForCompanyBranchFieldDataGroup)
                    {
                        new_resource.push(one_resource.clone());
                    }
                }
            }
        }

        new_resource
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subscribe {
    TableUserFieldName,
    TableUserFieldId,
    TableCompanyFieldName,
    TableCompanyFieldCurrency,
    TableCompanyBranchFieldName,
    TableCompanyBranchFieldCompanyBelong,
    TableAccessControlForCompanyFieldRole,
    TableAccessControlForCompanyFieldUser,
    TableAccessControlForCompanyFieldDataGroup,
    TableAccessControlForCompanyBranchFieldRole,
    TableAccessControlForCompanyBranchFieldUser,
    TableAccessControlForCompanyBranchFieldDataGroup,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
    Jwt(String),
    TableUserFieldName(String),
    TableUserFieldId(String),
    TableCompanyFieldName(String),
    TableCompanyFieldCurrency(db_types::Currency),
    TableCompanyBranchFieldName(String),
    TableCompanyBranchFieldCompanyBelong(db_types::UuidType),
    TableAccessControlForCompanyFieldRole(db_types::Role),
    TableAccessControlForCompanyFieldUser(db_types::UuidType),
    TableAccessControlForCompanyFieldDataGroup(db_types::UuidType),
    TableAccessControlForCompanyBranchFieldRole(db_types::Role),
    TableAccessControlForCompanyBranchFieldUser(db_types::UuidType),
    TableAccessControlForCompanyBranchFieldDataGroup(db_types::UuidType),
}

fn role_to_subscribe_mapping(roles: Vec<db_types::Role>) -> HashSet<Subscribe> {
    let mut subscribes = HashSet::with_capacity(200);

    for role in roles {
        match role {
            db_types::Role::Manager => {
                subscribes.insert(Subscribe::TableUserFieldName);
                subscribes.insert(Subscribe::TableUserFieldId);
                subscribes.insert(Subscribe::TableCompanyFieldName);
                subscribes.insert(Subscribe::TableCompanyFieldCurrency);
                subscribes.insert(Subscribe::TableCompanyBranchFieldName);
                subscribes.insert(Subscribe::TableCompanyBranchFieldCompanyBelong);
                subscribes.insert(Subscribe::TableAccessControlForCompanyFieldRole);
                subscribes.insert(Subscribe::TableAccessControlForCompanyFieldUser);
                subscribes.insert(Subscribe::TableAccessControlForCompanyFieldDataGroup);
            }
            db_types::Role::CoManager => todo!(),
        }
    }

    subscribes.shrink_to_fit();
    subscribes
}

pub struct AllRoles<Id: RowId> {
    pub companies: HashMap<
        Id, // company uuid
        HashMap<
            Id, // user uuid
            Vec<db_types::Role>,
        >,
    >,
    pub branches: HashMap<
        Id, // branch uuid
        HashMap<
            Id, // user uuid
            Vec<db_types::Role>,
        >,
    >,
}

pub struct AllSubscribes<Id: RowId> {
    pub companies: UserSubscribes<Id>,
    pub branches: UserSubscribes<Id>,
}

type UserSubscribes<Id> = HashMap<
    Id, // company uuid or branch
    HashMap<
        Id, // user uuid
        HashSet<Subscribe>,
    >,
>;

type UserSenders<Id, Mpsc: MultiProducerSingleConsumer> = HashMap<
    Id,                                            // user uuid
    HashMap<u64, Mpsc::Sender<Vec<ResourceInfo>>>, // because user may have multiple web socket connection
>;

type ListOfResources<Id> = HashMap<Id, Vec<ResourceInfo>>;

pub enum MessageToBroker<Id: RowId, Mpsc: MultiProducerSingleConsumer> {
    Subscribe {
        connection_id: u64,
        list_of_subscribtion: AllSubscribes<Id>,
        users_uuids: HashSet<Id>,
        sender_to_server: Mpsc::Sender<Vec<ResourceInfo>>,
    },
    Unsubscribe {
        connection_id: u64,
    },
    Publish {
        connection_id: u64,
        list_of_resources_for_company: ListOfResources<Id>,
        list_of_resources_for_branch: ListOfResources<Id>,
    },
}

pub(crate) struct SideEffects<Id: RowId> {
    pub(crate) authenticated_users: HashSet<Id>,
    pub(crate) resource_to_broadcast_for_company: ListOfResources<Id>,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources<Id>,
    pub(crate) users_to_resubscribe: HashSet<Id>,
}

impl<Id: RowId> Default for SideEffects<Id> {
    fn default() -> Self {
        Self {
            authenticated_users: Default::default(),
            resource_to_broadcast_for_company: Default::default(),
            resource_to_broadcast_for_branch: Default::default(),
            users_to_resubscribe: Default::default(),
        }
    }
}
