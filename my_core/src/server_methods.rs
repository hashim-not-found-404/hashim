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

pub struct ServerMethods<DB, Cli, Jwt, Authentication, F, Id, MPSC, RT>
where
    DB: Database<Client = Cli>,
    Cli: DBClient,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id, JsonWebToken = String>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    RT: Runtime,
{
    database: DB,
    client: PhantomData<Cli>,
    jwt: Jwt,
    authentication: PhantomData<Authentication>,
    functions: PhantomData<F>,
    rowid: PhantomData<Id>,
    mpsc: PhantomData<MPSC>,
    runtime: PhantomData<RT>,
    pub sender_to_broker: MPSC::Sender<MessageToBroker<Id, MPSC>>,
}

impl<DB, Cli, Jwt, Authentication, F, Id, MPSC, RT>
    ServerMethods<DB, Cli, Jwt, Authentication, F, Id, MPSC, RT>
where
    DB: Database<Client = Cli>,
    Cli: DBClient<RowId = Id, HashedPassword = Authentication>,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id, JsonWebToken = String>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    RT: Runtime,
{
    pub async fn new() -> Self {
        let (sender_to_broker, receiver_to_broker) = MPSC::channel();
        Self::broker_actor(receiver_to_broker);

        Self {
            database: DB::new().await,
            client: PhantomData,
            jwt: Jwt::new(),
            authentication: PhantomData,
            functions: PhantomData,
            rowid: PhantomData,
            mpsc: PhantomData,
            runtime: PhantomData,
            sender_to_broker,
        }
    }

    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let hashed_password = Authentication::sign_up(&input.password);

        let mut errr = sign_up::Error::default();

        let mut client = self.database.get_client().await?;
        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let is_new_user = txn.read_sign_up(&input.user_id).await?;

            if !is_new_user {
                errr.user_id = Some(sign_up::UserIdError::Duplicated);
                return Ok(Err(errr));
            }

            let user_uuid = txn
                .write_sign_up(&input.user_id, &hashed_password, &input.name)
                .await?;

            Ok(Ok(sign_up::Ok {
                jwt: self.jwt.sign(&user_uuid).into(),
            }))
        })()
        .await;

        if let Ok(Ok(_)) = &result {
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }

    pub async fn sign_in(
        &self,
        input: &sign_in::Input,
    ) -> Result<Result<(sign_in::Ok, Id), sign_in::Error>, DynamicError> {
        let mut errr = sign_in::Error::default();

        let mut client = self.database.get_client().await?;

        let (user_rowid, password_hash) = match client.read_sign_in(&input.user_id).await? {
            Some(p) => p,
            None => {
                errr.user_id = Some(sign_in::UserIdError::NotExist);
                return Ok(Err(errr));
            }
        };

        match Authentication::sign_in(&input.password, &password_hash) {
            true => {
                return Ok(Ok((
                    sign_in::Ok {
                        jwt: self.jwt.sign(&user_rowid).into(),
                    },
                    user_rowid,
                )));
            }
            false => {
                errr.password = Some(sign_in::PasswordError::WrongPassword);
                return Ok(Err(errr));
            }
        };
    }

    pub async fn create_company(
        &self,
        input: &create_company::Input,
    ) -> Result<Result<(create_company::Ok, Id), create_company::Error>, DynamicError> {
        let mut errr = create_company::Error::default();

        let user_uuid = match self.jwt.validate(input.jwt.clone()) {
            Some(user_uuid) => user_uuid,
            None => {
                errr.jwt = Some(JWTError::Invalid);
                return Ok(Err(errr));
            }
        };

        let nonce = match Id::try_from(&input.nonce) {
            Ok(nonce) => nonce,
            Err(_) => {
                errr.nonce = Some(NonceError::Invalid);
                return Ok(Err(errr));
            }
        };

        let mut client = self.database.get_client().await?;
        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let is_nonce_used = txn.read_create_company(&nonce).await?;

            if !check_nonce_if_valid::<Id>(&nonce, is_nonce_used) {
                errr.nonce = Some(NonceError::Invalid);
                return Ok(Err(errr));
            }

            let resources = txn
                .write_create_company(
                    &nonce,
                    &user_uuid,
                    &db_types::Role::Manager,
                    &input.company_name,
                    &input.currency,
                )
                .await?;

            Ok(Ok((create_company::Ok { resources }, user_uuid)))
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
        input: &create_company_branch::Input,
    ) -> Result<Result<(create_company_branch::Ok, Id), create_company_branch::Error>, DynamicError>
    {
        let mut errr = create_company_branch::Error::default();

        let user_uuid = match self.jwt.validate(input.jwt.clone()) {
            Some(user_uuid) => user_uuid,
            None => {
                errr.jwt = Some(JWTError::Invalid);
                return Ok(Err(errr));
            }
        };

        let nonce = match Id::try_from(&input.nonce) {
            Ok(nonce) => nonce,
            Err(_) => {
                errr.nonce = Some(NonceError::Invalid);
                return Ok(Err(errr));
            }
        };

        let company_belong = match Id::try_from(&input.company_belong) {
            Ok(company_belong) => company_belong,
            Err(_) => {
                errr.company_belong =
                    Some(create_company_branch::CompanyBelongError::IdInWrongFormat);
                return Ok(Err(errr));
            }
        };

        let mut client = self.database.get_client().await?;
        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let (is_nonce_used, is_company_exist, is_branch_name_used) = txn
                .read_create_company_branch(&nonce, &company_belong, &input.branch_name)
                .await?;

            if !check_nonce_if_valid::<Id>(&nonce, is_nonce_used) {
                errr.nonce = Some(NonceError::Invalid);
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

            let resources = txn
                .write_create_company_branch(
                    &nonce,
                    &company_belong,
                    &input.branch_name,
                    &input.location,
                    &input.currency,
                    &user_uuid,
                    &db_types::Role::Manager,
                )
                .await?;

            Ok(Ok((create_company_branch::Ok { resources }, user_uuid)))
        })()
        .await;

        if let Ok(Ok(_)) = &result {
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }

    pub async fn get_table_of_subscribed_data(
        &self,
        user_uuid: &Id,
    ) -> Result<AllSubscribesForUser<Id>, DynamicError> {
        let mut client = self.database.get_client().await?;
        let roles = client.read_roles_for_user(user_uuid).await?;

        let mut subs = AllSubscribesForUser {
            companies: HashMap::new(),
            branches: HashMap::new(),
        };

        for (company, role) in roles.companies {
            subs.companies
                .insert(company, role_to_subscribe_mapping(role));
        }

        for (branch, role) in roles.branches {
            subs.branches
                .insert(branch, role_to_subscribe_mapping(role));
        }

        Ok(subs)
    }

    pub fn broker_actor(receiver_to_broker: MPSC::Receiver<MessageToBroker<Id, MPSC>>) {
        RT::spawn_local(async move {
            let mut pool_of_pubsub_for_company: HashMap<
                Id, // company uuid
                HashMap<
                    Id, // user uuid
                    Vec<Subscribe>,
                >,
            > = HashMap::with_capacity(1000);

            let mut pool_of_pubsub_for_branch: HashMap<
                Id, // branch uuid
                HashMap<
                    Id, // user uuid
                    Vec<Subscribe>,
                >,
            > = HashMap::with_capacity(10000);

            let mut pool_of_server_facad_channels: HashMap<
                Id,                               // user uuid
                Vec<MPSC::Sender<Vec<Resource>>>, // because user may have multiple web socket connection
            > = HashMap::with_capacity(10000);

            loop {
                let message = receiver_to_broker.recv().await.unwrap();
                match message {
                    MessageToBroker::Subscribe {
                        user_uuid,
                        list_of_subscribtion_for_company,
                        list_of_subscribtion_for_branch,
                        sender_to_server,
                    } => {
                        let channels = pool_of_server_facad_channels.get_mut(&user_uuid);
                        match channels {
                            Some(channels) => {
                                channels.insert(channels.len(), sender_to_server.clone())
                            }
                            None => {
                                pool_of_server_facad_channels
                                    .insert(user_uuid.clone(), vec![sender_to_server.clone()]);
                            }
                        }

                        for (company, subscribes) in list_of_subscribtion_for_company {
                            let user_and_subscribes = pool_of_pubsub_for_company.get_mut(&company);
                            match user_and_subscribes {
                                Some(user_and_subscribes) => {
                                    user_and_subscribes.insert(user_uuid.clone(), subscribes);
                                }
                                None => {
                                    let mut user_and_subscribes = HashMap::new();
                                    user_and_subscribes.insert(user_uuid.clone(), subscribes);
                                    pool_of_pubsub_for_company.insert(company, user_and_subscribes);
                                }
                            }
                        }

                        for (branch, subscribes) in list_of_subscribtion_for_branch {
                            let user_and_subscribes = pool_of_pubsub_for_branch.get_mut(&branch);
                            match user_and_subscribes {
                                Some(user_and_subscribes) => {
                                    user_and_subscribes.insert(user_uuid.clone(), subscribes);
                                }
                                None => {
                                    let mut user_and_subscribes = HashMap::new();
                                    user_and_subscribes.insert(user_uuid.clone(), subscribes);
                                    pool_of_pubsub_for_branch.insert(branch, user_and_subscribes);
                                }
                            }
                        }
                    }
                    MessageToBroker::Unsubscribe { user_uuid } => {
                        pool_of_server_facad_channels.remove(&user_uuid);
                        // TODO : i have leake here i need to remove the company and branches if empty
                        for (_, users_and_subs) in pool_of_pubsub_for_company.iter_mut() {
                            users_and_subs.remove(&user_uuid);
                        }
                        for (_, users_and_subs) in pool_of_pubsub_for_branch.iter_mut() {
                            users_and_subs.remove(&user_uuid);
                        }
                    }
                    MessageToBroker::Publish {
                        list_of_resources_for_company,
                        list_of_resources_for_branch,
                    } => {
                        let mut resource_to_send: HashMap<
                            Id, // user uuid
                            Vec<Resource>,
                        > = HashMap::new();

                        for (company, resource) in list_of_resources_for_company {
                            let user_and_subscribe = pool_of_pubsub_for_company.get(&company);
                            match user_and_subscribe {
                                Some(user_and_subscribe) => {
                                    for (user_uuid, subscribe) in user_and_subscribe {
                                        let mut resource_for_user =
                                            resource_filtering_based_on_subscribe(
                                                subscribe, &resource,
                                            );

                                        let resource_to_append =
                                            resource_to_send.get_mut(user_uuid);
                                        match resource_to_append {
                                            Some(resource_to_append) => {
                                                resource_to_append.append(&mut resource_for_user)
                                            }
                                            None => {
                                                resource_to_send
                                                    .insert(user_uuid.clone(), resource_for_user);
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

                        for (branch, resource) in list_of_resources_for_branch {
                            let user_and_subscribe = pool_of_pubsub_for_branch.get(&branch);
                            match user_and_subscribe {
                                Some(user_and_subscribe) => {
                                    for (user_uuid, subscribe) in user_and_subscribe {
                                        let mut resource_for_user =
                                            resource_filtering_based_on_subscribe(
                                                subscribe, &resource,
                                            );

                                        let resource_to_append =
                                            resource_to_send.get_mut(user_uuid);
                                        match resource_to_append {
                                            Some(resource_to_append) => {
                                                resource_to_append.append(&mut resource_for_user)
                                            }
                                            None => {
                                                resource_to_send
                                                    .insert(user_uuid.clone(), resource_for_user);
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

// here dont contain data
pub enum Subscribe {
    // TODO
    CompanyCurrancy,
    CompanyName,
}
pub(crate) fn role_to_subscribe_mapping(roles: Vec<db_types::Role>) -> Vec<Subscribe> {
    let mut subscribes = Vec::with_capacity(200);

    for role in roles {
        match role {
            db_types::Role::Manager => {
                subscribes.push(Subscribe::CompanyCurrancy);
                subscribes.push(Subscribe::CompanyName);
            }
        }
    }

    subscribes.shrink_to_fit();
    subscribes
}

pub(crate) fn resource_filtering_based_on_subscribe(
    subscribe: &Vec<Subscribe>,
    resource: &Vec<Resource>,
) -> Vec<Resource> {
    todo!()
}

// here contain data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
    CompanyName(String),
    CompanyCurrency(db_types::Currency),
    RoleAtCompany(db_types::Role),
    UserThatHaveRole(db_types::RowIdType),
}
trait DataToResourceMapping {
    fn map_to_resource(&self) -> Vec<Resource>;
}

pub struct AllRolesForUser<Id: RowId> {
    pub companies: HashMap<Id, Vec<db_types::Role>>,
    pub branches: HashMap<Id, Vec<db_types::Role>>,
}

pub struct AllSubscribesForUser<Id: RowId> {
    pub companies: HashMap<Id, Vec<Subscribe>>,
    pub branches: HashMap<Id, Vec<Subscribe>>,
}

type SubscribedDataForCompany<Id> = HashMap<Id, Vec<Subscribe>>;
type SubscribedDataForBranch<Id> = HashMap<Id, Vec<Subscribe>>;
type ResourcesForCompany<Id> = HashMap<Id, Vec<Resource>>;
type ResourcesForBranch<Id> = HashMap<Id, Vec<Resource>>;

pub enum MessageToBroker<Id: RowId, MPSC: MultiProducerSingleConsumer> {
    Subscribe {
        user_uuid: Id,
        list_of_subscribtion_for_company: SubscribedDataForCompany<Id>,
        list_of_subscribtion_for_branch: SubscribedDataForBranch<Id>,
        sender_to_server: MPSC::Sender<Vec<Resource>>,
    },
    Unsubscribe {
        user_uuid: Id,
    },
    Publish {
        list_of_resources_for_company: ResourcesForCompany<Id>,
        list_of_resources_for_branch: ResourcesForBranch<Id>,
    },
}
