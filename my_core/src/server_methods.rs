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

pub struct ServerMethods<Db, Cli, Jwt, Auth, Rg, Id, Mpsc, Rt, De>
where
    Db: Database<Client = Cli>,
    Cli: DBClient,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Auth>,
    Jwt: JWT<UserId = Id, JsonWebToken = String>,
    Auth: HashedPassword,
    Rg: Regex,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    Rt: Runtime,
    De: Coding,
{
    _ph: PhantomData<(Cli, Auth, Rg, Id, Mpsc, Rt, De)>,
    database: Db,
    jwt: Jwt,
    pub sender_to_broker: Mpsc::Sender<MessageToBroker<Id, Mpsc>>,
}

impl<Db, Cli, Jwt, Auth, Rg, Id, Mpsc, Rt, De>
    ServerMethods<Db, Cli, Jwt, Auth, Rg, Id, Mpsc, Rt, De>
where
    Db: Database<Client = Cli> + 'static,
    Cli: DBClient<RowId = Id, HashedPassword = Auth> + 'static,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Auth>,
    Jwt: JWT<UserId = Id, JsonWebToken = String> + 'static,
    Auth: HashedPassword + 'static,
    Rg: Regex + 'static,
    Id: RowId + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    Rt: Runtime + 'static,
    De: Coding + 'static,
{
    pub async fn new() -> Self {
        let (sender_to_broker, receiver_to_broker) = Mpsc::channel();
        Self::broker_actor(receiver_to_broker);

        Self {
            _ph: PhantomData,
            database: Db::new().await,
            jwt: Jwt::new(),
            sender_to_broker,
        }
    }

    async fn sign_up(
        &self,
        client: &mut Cli,
        side_effects: &mut SideEffects<Id>,
        authenticated_users: &mut HashSet<Id>,
        input: &sign_up::Input,
    ) -> Result<sign_up::Result, DynamicError> {
        let mut errr = sign_up::Error::default();

        let new_uuid = match Id::try_from(&input.new_uuid) {
            Ok(new_uuid) => {
                authenticated_users.insert(new_uuid.clone());
                Some(new_uuid)
            }
            Err(_) => {
                errr.new_uuid = Some(RowIdError::Invalid);
                None
            }
        };

        if errr != sign_up::Error::default() {
            return Ok(Err(errr));
        }

        let hashed_password = Auth::sign_up(&input.password);

        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let new_uuid = new_uuid.unwrap();

            let (is_new_uuid_exist, is_user_id_exist) =
                txn.read_sign_up(&new_uuid, &input.user_id).await?;

            if is_new_uuid_exist {
                errr.new_uuid = Some(RowIdError::Duplicated);
            }

            if is_user_id_exist {
                errr.user_id = Some(sign_up::UserIdError::Duplicated);
            }

            if errr != sign_up::Error::default() {
                return Ok(Err(errr));
            }

            txn.write_sign_up(&new_uuid, &input.user_id, &hashed_password, &input.name)
                .await?;

            side_effects.resource_to_return.push(ResourceInfo {
                uuid: new_uuid.to_uuid(),
                resource: Resource::Jwt(self.jwt.sign(&new_uuid)),
            });
            side_effects.resource_to_return.push(ResourceInfo {
                uuid: new_uuid.to_uuid(),
                resource: Resource::UserId(input.user_id.clone()),
            });

            if let Some(name) = &input.name {
                side_effects.resource_to_return.push(ResourceInfo {
                    uuid: new_uuid.to_uuid(),
                    resource: Resource::UserName(name.clone()),
                });
            }

            Ok(Ok(sign_up::Ok))
        })()
        .await;

        if let Ok(Ok(_)) = &result {
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }

    async fn sign_in(
        &self,
        client: &mut Cli,
        side_effects: &mut SideEffects<Id>,
        authenticated_users: &mut HashSet<Id>,
        input: &sign_in::Input,
    ) -> Result<sign_in::Result, DynamicError> {
        let mut errr = sign_in::Error::default();

        let (user_rowid, password_hash) = match client.read_sign_in(&input.user_id).await? {
            Some(p) => p,
            None => {
                errr.user_id = Some(sign_in::UserIdError::NotExist);
                return Ok(Err(errr));
            }
        };

        match Auth::sign_in(&input.password, &password_hash) {
            true => {
                authenticated_users.insert(user_rowid.clone());
                side_effects.users_to_resubscribe.insert(user_rowid.clone());
                side_effects.resource_to_return.push(ResourceInfo {
                    uuid: user_rowid.to_uuid(),
                    resource: Resource::UserId(input.user_id.clone()),
                });
                side_effects.resource_to_return.push(ResourceInfo {
                    uuid: user_rowid.to_uuid(),
                    resource: Resource::Jwt(self.jwt.sign(&user_rowid)),
                });
                return Ok(Ok(sign_in::Ok));
            }
            false => {
                errr.password = Some(sign_in::PasswordError::WrongPassword);
                return Ok(Err(errr));
            }
        };
    }

    async fn create_company(
        &self,
        client: &mut Cli,
        side_effects: &mut SideEffects<Id>,
        authenticated_users: &mut HashSet<Id>,
        input: &create_company::Input,
    ) -> Result<create_company::Result, DynamicError> {
        let mut errr = create_company::Error::default();

        let new_uuid = match Id::try_from(&input.new_uuid) {
            Ok(new_uuid) => Some(new_uuid),
            Err(_) => {
                errr.new_uuid = Some(RowIdError::Invalid);
                None
            }
        };

        let user_uuid = match Id::try_from(&input.user_uuid) {
            Ok(user_uuid) => {
                if authenticated_users.get(&user_uuid).is_none() {
                    errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                };
                Some(user_uuid)
            }
            Err(_) => {
                errr.user_uuid = Some(UserUuidError::Invalid);
                None
            }
        };

        if errr != create_company::Error::default() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let new_uuid = new_uuid.unwrap();
            let user_uuid = user_uuid.unwrap();

            let is_new_uuid_used = txn.read_create_company(&new_uuid).await?;

            if is_new_uuid_used {
                errr.new_uuid = Some(RowIdError::Duplicated);
                return Ok(Err(errr));
            }

            const ROLE: db_types::Role = db_types::Role::Manager;

            txn.write_create_company(
                &new_uuid,
                &user_uuid,
                &ROLE,
                &input.company_name,
                &input.currency,
            )
            .await?;

            side_effects.users_to_resubscribe.insert(user_uuid);

            side_effects.resource_to_broadcast_for_company.insert_push(
                new_uuid.clone(),
                ResourceInfo {
                    uuid: new_uuid.to_uuid(),
                    resource: server_methods::Resource::CompanyName(input.company_name.clone()),
                },
            );
            side_effects.resource_to_broadcast_for_company.insert_push(
                new_uuid.clone(),
                ResourceInfo {
                    uuid: new_uuid.to_uuid(),
                    resource: server_methods::Resource::CompanyCurrency(input.currency.clone()),
                },
            );
            side_effects.resource_to_broadcast_for_company.insert_push(
                new_uuid.clone(),
                ResourceInfo {
                    uuid: new_uuid.to_uuid(),
                    resource: server_methods::Resource::RoleAtCompany(ROLE),
                },
            );
            side_effects.resource_to_broadcast_for_company.insert_push(
                new_uuid.clone(),
                ResourceInfo {
                    uuid: new_uuid.to_uuid(),
                    resource: server_methods::Resource::UserThatHaveRole(input.user_uuid.clone()),
                },
            );
            side_effects.resource_to_broadcast_for_company.insert_push(
                new_uuid.clone(),
                ResourceInfo {
                    uuid: new_uuid.to_uuid(),
                    resource: server_methods::Resource::CompanyThatHaveUserRole(new_uuid.to_uuid()),
                },
            );

            Ok(Ok(create_company::Ok))
        })()
        .await;

        if let Ok(Ok(_)) = &result {
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }

    pub async fn create_company_branch(
        &self,
        client: &mut Cli,
        side_effects: &mut SideEffects<Id>,
        authenticated_users: &mut HashSet<Id>,
        input: &create_company_branch::Input,
    ) -> Result<create_company_branch::Result, DynamicError> {
        let mut errr = create_company_branch::Error::default();

        let new_uuid = match Id::try_from(&input.new_uuid) {
            Ok(new_uuid) => Some(new_uuid),
            Err(_) => {
                errr.new_uuid = Some(RowIdError::Invalid);
                None
            }
        };

        let user_uuid = match Id::try_from(&input.user_uuid) {
            Ok(user_uuid) => {
                if authenticated_users.get(&user_uuid).is_none() {
                    errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                };
                Some(user_uuid)
            }
            Err(_) => {
                errr.user_uuid = Some(UserUuidError::Invalid);
                None
            }
        };

        let company_belong = match Id::try_from(&input.company_belong) {
            Ok(company_belong) => Some(company_belong),
            Err(_) => {
                errr.company_belong =
                    Some(create_company_branch::CompanyBelongError::IdInWrongFormat);
                None
            }
        };

        if errr != create_company_branch::Error::default() {
            return Ok(Err(errr));
        }

        let new_uuid = new_uuid.unwrap();
        let user_uuid = user_uuid.unwrap();
        let company_belong = company_belong.unwrap();

        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            todo!("get the role of the user to check it");
            let (is_new_uuid_used, is_company_exist, is_branch_name_used) = txn
                .read_create_company_branch(&new_uuid, &company_belong, &input.branch_name)
                .await?;

            if is_new_uuid_used {
                errr.new_uuid = Some(RowIdError::Duplicated);
            }

            if is_company_exist {
                errr.company_belong = Some(create_company_branch::CompanyBelongError::NotExist);
            }

            if is_branch_name_used {
                errr.branch_name = Some(create_company_branch::BranchNameError::Duplicated);
            }

            if !input.location.is_valid() {
                errr.location = Some(create_company_branch::LocationError::Invalid);
            }

            if errr != create_company_branch::Error::default() {
                return Ok(Err(errr));
            }

            txn.write_create_company_branch(
                &new_uuid,
                &company_belong,
                &input.branch_name,
                &input.location,
                &input.currency,
                &user_uuid,
                &db_types::Role::Manager,
            )
            .await?;

            side_effects.users_to_resubscribe.insert(user_uuid);

            todo!("add to the resource");
            Ok(Ok(create_company_branch::Ok))
        })()
        .await;

        if let Ok(Ok(_)) = &result {
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }

    async fn push_data(
        &self,
        client: &mut Cli,
        side_effects: &mut SideEffects<Id>,
        input: &push_data::Input,
    ) -> Result<push_data::Result, DynamicError> {
        let mut the_return_result = push_data::Result {
            jwts: Vec::with_capacity(input.jwts.len()),
            nonce: Ok(()),
            operations: Vec::with_capacity(input.operations.len()),
        };

        let mut is_there_error = false;
        let mut authenticated_users = HashSet::with_capacity(input.jwts.len());

        for jwt in &input.jwts {
            match self.jwt.validate(jwt.clone()) {
                Some(user_uuid) => {
                    authenticated_users.insert(user_uuid);
                }
                None => {
                    the_return_result.jwts.push(Err(JWTError::Invalid));

                    is_there_error = true;
                }
            }
        }

        let nonce = match Id::try_from(&input.nonce) {
            Ok(nonce) => nonce,
            Err(_) => {
                the_return_result.nonce = Err(NonceError::Invalid);
                return Ok(the_return_result);
            }
        };

        let is_nonce_used = client.write_nonce_if_not_used(&nonce).await?;

        if !check_nonce_if_valid::<Id>(&nonce, is_nonce_used) {
            the_return_result.nonce = Err(NonceError::Invalid);
        }

        if is_there_error {
            return Ok(the_return_result);
        }

        for transaction in &input.operations {
            let result = match &transaction.operation {
                push_data::OperationsInput::SignUp(input) => {
                    let result = self
                        .sign_up(client, side_effects, &mut authenticated_users, input)
                        .await?;
                    push_data::OperationsResult::SignUp(result)
                }
                push_data::OperationsInput::SignIn(input) => {
                    let result = self
                        .sign_in(client, side_effects, &mut authenticated_users, input)
                        .await?;
                    push_data::OperationsResult::SignIn(result)
                }
                push_data::OperationsInput::CreateCompany(input) => {
                    let result = self
                        .create_company(client, side_effects, &mut authenticated_users, input)
                        .await?;
                    push_data::OperationsResult::CreateCompany(result)
                }
                push_data::OperationsInput::CreateCompanyBranch(input) => {
                    let result = self
                        .create_company_branch(
                            client,
                            side_effects,
                            &mut authenticated_users,
                            input,
                        )
                        .await?;
                    push_data::OperationsResult::CreateCompanyBranch(result)
                }
            };

            the_return_result.operations.push(push_data::Txn {
                txn_number: transaction.txn_number,
                operation: result,
            });
        }

        return Ok(the_return_result);
    }

    pub async fn get_table_of_subscribed_data(
        &self,
        client: &mut Cli,
        users_uuids: &HashSet<Id>,
    ) -> Result<AllSubscribes<Id>, DynamicError> {
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

    pub fn server_actor<Ws: WSServer + 'static>(self: Arc<Self>, mut session: Ws) {
        Rt::spawn_local(async move {
            let mut sender_to_broker = self.sender_to_broker.clone();

            let (sender_to_server, mut receiver_to_server) = Mpsc::channel::<Vec<ResourceInfo>>();

            loop {
                let result = Rt::select(session.receive(), receiver_to_server.recv()).await;
                match result {
                    Either::One(msg) => {
                        let msg = match msg {
                            Ok(msg) => msg,
                            Err(_) => break,
                        };

                        match msg {
                            WSMessage::Close => break,
                            WSMessage::Binary(received_data) => {
                                let input = match De::decode::<messages::FromClient>(&received_data)
                                {
                                    Ok(ok) => ok,
                                    Err(_) => {
                                        if session
                                            .send_bin(De::encode(&messages::FromServer::Error(
                                                HashimError::InvalidDataFormat,
                                            )))
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
                                            .send_bin(De::encode(&messages::FromServer::Error(
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

                                let mut side_effects = SideEffects::<Id>::default();
                                match self.push_data(&mut client, &mut side_effects, &input).await {
                                    Ok(ok) => {
                                        if session
                                            .send_bin(De::encode(&messages::FromServer::PushData(
                                                ok,
                                            )))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        if session
                                            .send_bin(De::encode(&messages::FromServer::Error(
                                                HashimError::InternalServerError,
                                            )))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                }
                                todo!("make the server send all data to user");

                                if !side_effects.resource_to_return.is_empty() {
                                    if session
                                        .send_bin(De::encode(&messages::FromServer::Resources(
                                            side_effects.resource_to_return,
                                        )))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }

                                if !side_effects.users_to_resubscribe.is_empty() {
                                    let subs = match self
                                        .get_table_of_subscribed_data(
                                            &mut client,
                                            &side_effects.users_to_resubscribe,
                                        )
                                        .await
                                    {
                                        Ok(ok) => ok,
                                        Err(_) => {
                                            if session
                                                .send_bin(De::encode(&messages::FromServer::Error(
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

                                    sender_to_broker
                                        .send(server_methods::MessageToBroker::Subscribe {
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
                            .send_bin(De::encode(&messages::FromServer::Resources(resource)))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }

            session.close().await.unwrap();
        });
    }

    pub fn broker_actor(mut receiver_to_broker: Mpsc::Receiver<MessageToBroker<Id, Mpsc>>) {
        Rt::spawn_local(async move {
            let mut pool_of_pubsub_for_company: UserSubscribes<Id> = HashMap::with_capacity(1000);
            let mut pool_of_pubsub_for_branch: UserSubscribes<Id> = HashMap::with_capacity(10000);
            let mut pool_of_server_facad_channels: UserSenders<Id, Mpsc> =
                HashMap::with_capacity(10000);

            loop {
                let message = receiver_to_broker.recv().await.unwrap();
                match message {
                    MessageToBroker::Subscribe {
                        list_of_subscribtion,
                        users_uuids,
                        sender_to_server,
                    } => {
                        for user_uuid in users_uuids {
                            let channels = pool_of_server_facad_channels.get_mut(&user_uuid);
                            match channels {
                                Some(channels) => channels.push(sender_to_server.clone()),
                                None => {
                                    pool_of_server_facad_channels
                                        .insert(user_uuid, vec![sender_to_server.clone()]);
                                }
                            }
                        }

                        merge_subscribes(
                            &mut pool_of_pubsub_for_company,
                            list_of_subscribtion.companies,
                        );

                        merge_subscribes(
                            &mut pool_of_pubsub_for_branch,
                            list_of_subscribtion.branches,
                        );
                    }
                    MessageToBroker::Unsubscribe { user_uuid } => {
                        pool_of_server_facad_channels.remove(&user_uuid);
                        // TODO : i have leake here i need to remove the company and branches if empty
                        unsubscribe(&mut pool_of_pubsub_for_company, &user_uuid);
                        unsubscribe(&mut pool_of_pubsub_for_branch, &user_uuid);
                    }
                    MessageToBroker::Publish {
                        list_of_resources_for_company,
                        list_of_resources_for_branch,
                    } => {
                        let mut resource_to_send: ListOfResources<Id /* user id */> =
                            HashMap::new();

                        map_resource_to_subscribes(
                            &pool_of_pubsub_for_company,
                            list_of_resources_for_company,
                            &mut resource_to_send,
                        );

                        map_resource_to_subscribes(
                            &pool_of_pubsub_for_branch,
                            list_of_resources_for_branch,
                            &mut resource_to_send,
                        );

                        for (user_uuid, resource) in resource_to_send {
                            let channels = pool_of_server_facad_channels.get_mut(&user_uuid);

                            match channels {
                                Some(channels) => {
                                    let mut index = 0;
                                    while index < channels.len() {
                                        if channels[index].send(resource.clone()).await.is_err() {
                                            channels.remove(index);
                                        } else {
                                            index += 1;
                                        }
                                    }

                                    if channels.len() == 0 {
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

pub enum WSMessage {
    Binary(Vec<u8>),
    Close,
}

pub trait WSServer {
    async fn send_bin(&mut self, bin: Vec<u8>) -> Result<(), DynamicError>;
    async fn receive(&mut self) -> Result<WSMessage, DynamicError>;
    async fn close(self) -> Result<(), DynamicError>;
}

fn map_resource_to_subscribes<Id: RowId>(
    pool_of_pubsub: &UserSubscribes<Id>,
    list_of_resources: ListOfResources<Id>,
    resource_to_send: &mut ListOfResources<Id>,
) {
    for (company, resource) in list_of_resources {
        let user_and_subscribe = pool_of_pubsub.get(&company);
        match user_and_subscribe {
            Some(user_and_subscribe) => {
                for (user_uuid, subscribe) in user_and_subscribe {
                    let mut resource_for_user =
                        resource_filtering_based_on_subscribe(subscribe, &resource);

                    let resource_to_append = resource_to_send.get_mut(user_uuid);
                    match resource_to_append {
                        Some(resource_to_append) => {
                            resource_to_append.append(&mut resource_for_user)
                        }
                        None => {
                            resource_to_send.insert(user_uuid.clone(), resource_for_user);
                        }
                    }
                }
            }
            None => {
                dbg!("there is some problem here this should not happen");
                continue;
            }
        }
    }
}

fn unsubscribe<Id: RowId>(pool_of_pubsub: &mut UserSubscribes<Id>, user_uuid: &Id) {
    for (_, users_and_subs) in pool_of_pubsub.iter_mut() {
        users_and_subs.remove(user_uuid);
    }
}

fn merge_subscribes<Id: RowId>(
    pool_of_pubsub: &mut UserSubscribes<Id>,
    list_of_subscribtion: UserSubscribes<Id>,
) {
    for (company, users_subscribes) in list_of_subscribtion {
        let users_and_subscribes = pool_of_pubsub.get_mut(&company);
        match users_and_subscribes {
            Some(users_and_subscribes) => {
                for (user_uuid, subscribes) in users_subscribes {
                    users_and_subscribes.insert(user_uuid.clone(), subscribes);
                }
            }
            None => {
                let mut users_and_subscribes = HashMap::new();

                for (user_uuid, subscribes) in users_subscribes {
                    users_and_subscribes.insert(user_uuid, subscribes);
                }

                pool_of_pubsub.insert(company, users_and_subscribes);
            }
        }
    }
}

// here dont contain data
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Subscribe {
    UserName,
    UserId,
    CompanyName,
    CompanyCurrency,
    RoleAtCompany,
    UserThatHaveRole,
    CompanyThatHaveUserRole,
}

pub(crate) fn role_to_subscribe_mapping(roles: Vec<db_types::Role>) -> HashSet<Subscribe> {
    let mut subscribes = HashSet::with_capacity(200);

    for role in roles {
        match role {
            db_types::Role::Manager => {
                subscribes.insert(Subscribe::UserName);
                subscribes.insert(Subscribe::UserId);
                subscribes.insert(Subscribe::CompanyName);
                subscribes.insert(Subscribe::CompanyCurrency);
                subscribes.insert(Subscribe::RoleAtCompany);
                subscribes.insert(Subscribe::UserThatHaveRole);
                subscribes.insert(Subscribe::CompanyThatHaveUserRole);
            }
        }
    }

    subscribes.shrink_to_fit();
    subscribes
}

pub(crate) fn resource_filtering_based_on_subscribe(
    subscribe: &HashSet<Subscribe>,
    resource: &Vec<ResourceInfo>,
) -> Vec<ResourceInfo> {
    let mut new_resource = Vec::new();

    for one_resource in resource {
        match one_resource.resource {
            Resource::Jwt(_) => {}
            Resource::UserName(_) => {
                if subscribe.contains(&Subscribe::UserName) {
                    new_resource.push(one_resource.clone());
                }
            }
            Resource::UserId(_) => {
                if subscribe.contains(&Subscribe::UserId) {
                    new_resource.push(one_resource.clone());
                }
            }
            Resource::CompanyName(_) => {
                if subscribe.contains(&Subscribe::CompanyName) {
                    new_resource.push(one_resource.clone());
                }
            }
            Resource::CompanyCurrency(_) => {
                if subscribe.contains(&Subscribe::CompanyCurrency) {
                    new_resource.push(one_resource.clone());
                }
            }
            Resource::RoleAtCompany(_) => {
                if subscribe.contains(&Subscribe::RoleAtCompany) {
                    new_resource.push(one_resource.clone());
                }
            }
            Resource::UserThatHaveRole(_) => {
                if subscribe.contains(&Subscribe::UserThatHaveRole) {
                    new_resource.push(one_resource.clone());
                }
            }
            Resource::CompanyThatHaveUserRole(_) => {
                if subscribe.contains(&Subscribe::CompanyThatHaveUserRole) {
                    new_resource.push(one_resource.clone());
                }
            }
        }
    }

    new_resource
}

// here contain data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
    Jwt(String),
    UserName(String),
    UserId(String),
    CompanyName(String),
    CompanyCurrency(db_types::Currency),
    RoleAtCompany(db_types::Role),
    UserThatHaveRole(db_types::UuidType),
    CompanyThatHaveUserRole(db_types::UuidType),
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
    Id,                                   // user uuid
    Vec<Mpsc::Sender<Vec<ResourceInfo>>>, // because user may have multiple web socket connection
>;

type ListOfResources<Id> = HashMap<Id, Vec<ResourceInfo>>;

pub enum MessageToBroker<Id: RowId, Mpsc: MultiProducerSingleConsumer> {
    Subscribe {
        list_of_subscribtion: AllSubscribes<Id>,
        users_uuids: HashSet<Id>,
        sender_to_server: Mpsc::Sender<Vec<ResourceInfo>>,
    },
    Unsubscribe {
        user_uuid: Id,
    },
    Publish {
        list_of_resources_for_company: ListOfResources<Id>,
        list_of_resources_for_branch: ListOfResources<Id>,
    },
}

pub struct SideEffects<Id: RowId> {
    resource_to_broadcast_for_company: ListOfResources<Id>,
    resource_to_broadcast_for_branch: ListOfResources<Id>,
    resource_to_return: Vec<ResourceInfo>,
    users_to_resubscribe: HashSet<Id>,
}

impl<Id: RowId> Default for SideEffects<Id> {
    fn default() -> Self {
        Self {
            resource_to_broadcast_for_company: Default::default(),
            resource_to_broadcast_for_branch: Default::default(),
            resource_to_return: Default::default(),
            users_to_resubscribe: Default::default(),
        }
    }
}

pub trait ExtendHashMap<K, V> {
    fn insert_push(&mut self, k: K, v: V);
}

impl<K: Eq + Hash, V> ExtendHashMap<K, V> for HashMap<K, Vec<V>> {
    fn insert_push(&mut self, k: K, v: V) {
        self.entry(k).or_insert_with(Vec::new).push(v);
    }
}
