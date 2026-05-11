use crate::prelude::*;

pub struct ServerMethods<DB, Cli, Jwt, Authentication, F, Id>
where
    DB: Database<Client = Cli>,
    Cli: DBClient,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId,
{
    database: DB,
    client: PhantomData<Cli>,
    jwt: Jwt,
    authentication: PhantomData<Authentication>,
    functions: PhantomData<F>,
    rowid: PhantomData<Id>,
}

impl<DB, Cli, Jwt, Authentication, F, Id> ServerMethods<DB, Cli, Jwt, Authentication, F, Id>
where
    DB: Database<Client = Cli>,
    Cli: DBClient<RowId = Id, HashedPassword = Authentication>,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId,
{
    pub fn new(database: DB, jwt: Jwt) -> Self {
        Self {
            database,
            client: PhantomData::<Cli>,
            jwt,
            authentication: PhantomData::<Authentication>,
            functions: PhantomData::<F>,
            rowid: PhantomData::<Id>,
        }
    }

    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let hashed_password = Authentication::sign_up(&input.password);

        let mut errr = sign_up::Error {
            user_id: None,
            name: None,
        };

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

    pub async fn sign_in(&self, input: &sign_in::Input) -> Result<sign_in::Result, DynamicError> {
        let mut errr = sign_in::Error {
            user_id: None,
            password: None,
        };

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

    pub async fn get_table_of_subscribed_data(
        &self,
        user_uuid: &Id,
    ) -> Result<SubscribedData<Id>, DynamicError> {
        let mut client = self.database.get_client().await?;
        let roles = client.read_roles_for_user(user_uuid).await?;

        let mut subs: SubscribedData<Id> = HashMap::with_capacity(roles.len());
        for (company, role) in roles {
            subs.insert(company, role_to_subscribe_mapping(role));
        }
        subs.shrink_to_fit();
        Ok(subs)
    }
}

// here dont contain data
pub enum Subscribe {}
pub(crate) fn role_to_subscribe_mapping(role: db_types::Role) -> Vec<Subscribe> {
    todo!()
}

pub(crate) fn resource_filtering_based_on_subscribe(
    subscribe: &Vec<Subscribe>,
    resource: &Vec<Resource>,
) -> Vec<Resource> {
    todo!()
}

// here dont data
#[derive(Clone)]
pub enum Resource {}
trait DataToResourceMapping {
    fn map_to_resource(&self) -> Vec<Resource>;
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub enum CompanyOrBranch<Id: RowId> {
    Company(Id),
    Branch(Id),
}

type SubscribedData<Id> = HashMap<CompanyOrBranch<Id>, Vec<Subscribe>>;
type ResourcesToShare<Id> = HashMap<CompanyOrBranch<Id>, Vec<Resource>>;

pub enum MessageToBroker<Id: RowId, MPSC: MultiProducerSingleConsumer> {
    Subscribe(Id, SubscribedData<Id>, MPSC::Sender<Vec<Resource>>),
    Publish(ResourcesToShare<Id>),
}

pub fn broker_actor<
    RT: Runtime,
    MPSC: MultiProducerSingleConsumer + 'static,
    Id: RowId + 'static,
>(
    receiver_to_broker: MPSC::Receiver<MessageToBroker<Id, MPSC>>,
) {
    RT::spawn(async move {
        let mut pool_of_pubsub: HashMap<
            CompanyOrBranch<Id>, // company uuid
            HashMap<
                Id, // user uuid
                Vec<Subscribe>,
            >,
        > = HashMap::with_capacity(100000);

        let mut pool_of_server_facad_channels: HashMap<
            Id,                               // user uuid
            Vec<MPSC::Sender<Vec<Resource>>>, // because user may have multiple web socket connection
        > = HashMap::with_capacity(10000);

        loop {
            let message = receiver_to_broker.recv().await.unwrap();
            match message {
                MessageToBroker::Subscribe(
                    user_uuid,
                    list_of_subscribtion,
                    channel_to_send_to_facad,
                ) => {
                    let channels = pool_of_server_facad_channels.get_mut(&user_uuid);

                    match channels {
                        Some(channels) => {
                            channels.insert(channels.len(), channel_to_send_to_facad.clone())
                        }
                        None => {
                            pool_of_server_facad_channels
                                .insert(user_uuid.clone(), vec![channel_to_send_to_facad.clone()]);
                        }
                    }

                    for (company, subscribes) in list_of_subscribtion {
                        let user_and_subscribes = pool_of_pubsub.get_mut(&company);
                        match user_and_subscribes {
                            Some(user_and_subscribes) => {
                                user_and_subscribes.insert(user_uuid.clone(), subscribes);
                            }
                            None => {
                                let mut user_and_subscribes = HashMap::new();
                                user_and_subscribes.insert(user_uuid.clone(), subscribes);
                                pool_of_pubsub.insert(company, user_and_subscribes);
                            }
                        }
                    }
                }
                MessageToBroker::Publish(list_of_resource) => {
                    let mut resource_to_send: HashMap<
                        Id, // user uuid
                        Vec<Resource>,
                    > = HashMap::new();

                    for (company, resource) in list_of_resource {
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
