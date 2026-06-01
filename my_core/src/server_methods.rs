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
    _ph: PhantomData<(Cli, Authentication, F, Id, MPSC, RT)>,
    database: DB,
    jwt: Jwt,
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
            _ph: PhantomData,
            database: DB::new().await,
            jwt: Jwt::new(),
            sender_to_broker,
        }
    }

    pub async fn sign_up(
        &self,
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

        let hashed_password = Authentication::sign_up(&input.password);

        let mut client = self.database.get_client().await?;
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

            Ok(Ok(sign_up::Ok {
                jwt: self.jwt.sign(&new_uuid).into(),
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
        authenticated_users: &mut HashSet<Id>,
        users_to_resubscribe: &mut HashSet<Id>,
        input: &sign_in::Input,
    ) -> Result<sign_in::Result, DynamicError> {
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
                authenticated_users.insert(user_rowid.clone());
                users_to_resubscribe.insert(user_rowid.clone());
                return Ok(Ok(sign_in::Ok {
                    jwt: self.jwt.sign(&user_rowid).into(),
                }));
            }
            false => {
                errr.password = Some(sign_in::PasswordError::WrongPassword);
                return Ok(Err(errr));
            }
        };
    }

    pub async fn create_company(
        &self,
        resources: &mut HashSet<ResourceInfo>,
        authenticated_users: &mut HashSet<Id>,
        users_to_resubscribe: &mut HashSet<Id>,
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

        let mut client = self.database.get_client().await?;
        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let new_uuid = new_uuid.unwrap();
            let user_uuid = user_uuid.unwrap();

            let is_new_uuid_used = txn.read_create_company(&new_uuid).await?;

            if is_new_uuid_used {
                errr.new_uuid = Some(RowIdError::Duplicated);
                return Ok(Err(errr));
            }

            txn.write_create_company(
                resources,
                &new_uuid,
                &user_uuid,
                &db_types::Role::Manager,
                &input.company_name,
                &input.currency,
            )
            .await?;

            users_to_resubscribe.insert(user_uuid);
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
        resources: &mut HashSet<ResourceInfo>,
        authenticated_users: &mut HashSet<Id>,
        users_to_resubscribe: &mut HashSet<Id>,
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

        let mut client = self.database.get_client().await?;
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
                resources,
                &new_uuid,
                &company_belong,
                &input.branch_name,
                &input.location,
                &input.currency,
                &user_uuid,
                &db_types::Role::Manager,
            )
            .await?;

            users_to_resubscribe.insert(user_uuid);
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

    pub async fn push_data(
        &self,
        resources: &mut HashSet<ResourceInfo>,
        users_to_resubscribe: &mut HashSet<Id>,
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

        let mut client = self.database.get_client().await?;
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
                    let result = self.sign_up(&mut authenticated_users, input).await?;
                    push_data::OperationsResult::SignUp(result)
                }
                push_data::OperationsInput::SignIn(input) => {
                    let result = self
                        .sign_in(&mut authenticated_users, users_to_resubscribe, input)
                        .await?;
                    push_data::OperationsResult::SignIn(result)
                }
                push_data::OperationsInput::CreateCompany(input) => {
                    let result = self
                        .create_company(
                            resources,
                            &mut authenticated_users,
                            users_to_resubscribe,
                            input,
                        )
                        .await?;
                    push_data::OperationsResult::CreateCompany(result)
                }
                push_data::OperationsInput::CreateCompanyBranch(input) => {
                    let result = self
                        .create_company_branch(
                            resources,
                            &mut authenticated_users,
                            users_to_resubscribe,
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
        users_uuids: &HashSet<Id>,
    ) -> Result<AllSubscribes<Id>, DynamicError> {
        let mut client = self.database.get_client().await?;
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

    pub fn broker_actor(receiver_to_broker: MPSC::Receiver<MessageToBroker<Id, MPSC>>) {
        RT::spawn_local(async move {
            let mut pool_of_pubsub_for_company: CompanyUserSubscribes<Id> =
                HashMap::with_capacity(1000);

            let mut pool_of_pubsub_for_branch: BranchUserSubscribes<Id> =
                HashMap::with_capacity(10000);

            let mut pool_of_server_facad_channels: UserSenders<Id, MPSC> =
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
                        let mut resource_to_send: HashMap<
                            Id, // user uuid
                            Vec<ResourceInfo>,
                        > = HashMap::new();

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

fn map_resource_to_subscribes<Id: RowId>(
    pool_of_pubsub: &HashMap<Id, HashMap<Id, Vec<Subscribe>>>,
    list_of_resources: HashMap<Id, Vec<ResourceInfo>>,
    resource_to_send: &mut HashMap<Id, Vec<ResourceInfo>>,
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

fn unsubscribe<Id: RowId>(
    pool_of_pubsub: &mut HashMap<Id, HashMap<Id, Vec<Subscribe>>>,
    user_uuid: &Id,
) {
    for (_, users_and_subs) in pool_of_pubsub.iter_mut() {
        users_and_subs.remove(user_uuid);
    }
}

fn merge_subscribes<Id: RowId>(
    pool_of_pubsub: &mut HashMap<Id, HashMap<Id, Vec<Subscribe>>>,
    list_of_subscribtion: HashMap<Id, HashMap<Id, Vec<Subscribe>>>,
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
    resource: &Vec<ResourceInfo>,
) -> Vec<ResourceInfo> {
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
    pub companies: CompanyUserSubscribes<Id>,
    pub branches: BranchUserSubscribes<Id>,
}

type CompanyUserSubscribes<Id> = HashMap<
    Id, // company uuid
    HashMap<
        Id, // user uuid
        Vec<Subscribe>,
    >,
>;

type BranchUserSubscribes<Id> = HashMap<
    Id, // branch uuid
    HashMap<
        Id, // user uuid
        Vec<Subscribe>,
    >,
>;

type UserSenders<Id, MPSC: MultiProducerSingleConsumer> = HashMap<
    Id,                                   // user uuid
    Vec<MPSC::Sender<Vec<ResourceInfo>>>, // because user may have multiple web socket connection
>;

type ResourcesForCompany<Id> = HashMap<Id, Vec<ResourceInfo>>;
type ResourcesForBranch<Id> = HashMap<Id, Vec<ResourceInfo>>;

pub enum MessageToBroker<Id: RowId, MPSC: MultiProducerSingleConsumer> {
    Subscribe {
        list_of_subscribtion: AllSubscribes<Id>,
        users_uuids: HashSet<Id>,
        sender_to_server: MPSC::Sender<Vec<ResourceInfo>>,
    },
    Unsubscribe {
        user_uuid: Id,
    },
    Publish {
        list_of_resources_for_company: ResourcesForCompany<Id>,
        list_of_resources_for_branch: ResourcesForBranch<Id>,
    },
}
