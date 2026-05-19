use crate::prelude::*;

pub trait Signal {
    type T;
    fn read(&self) -> Self::T;
    fn set(&self, v: Self::T);
}

pub struct State<
    WA: WAMP<Sender<DynamicError> = MPSC::Sender<DynamicError>> + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    CH: CacheIO + 'static,
    Id: RowId + 'static,
    RN: RandomNumber + 'static,
    SigString: Signal<T = String> + 'static,
    SigBool: Signal<T = bool> + 'static,
    SigExternalError: Signal<T = String> + 'static,
    SigCurrency: Signal<T = db_types::Currency> + 'static,
    SigLocation: Signal<T = db_types::Location> + 'static,
> {
    // here for the app logic
    id: PhantomData<Id>,
    random_number: PhantomData<RN>,
    sig_string: PhantomData<SigString>,
    sig_currency: PhantomData<SigCurrency>,
    sig_location: PhantomData<SigLocation>,

    routs: Arc<client::RoutsForClientSide<WA, RT, MPSC, CH>>,
    jwt: RwLock<Option<String>>,

    // here every field to display
    // here is global state
    pub is_signed_in: SigBool,
    // here is feature state
    pub external_errors: SigExternalError,
}

impl<
    WA: WAMP<Sender<DynamicError> = MPSC::Sender<DynamicError>> + 'static,
    RT: Runtime,
    MPSC: MultiProducerSingleConsumer,
    CH: CacheIO,
    Id: RowId,
    RN: RandomNumber,
    // signals
    SigString: Signal<T = String>,
    SigBool: Signal<T = bool> + Default,
    SigExternalError: Signal<T = String> + Default,
    SigCurrency: Signal<T = db_types::Currency>,
    SigLocation: Signal<T = db_types::Location>,
> State<WA, RT, MPSC, CH, Id, RN, SigString, SigBool, SigExternalError, SigCurrency, SigLocation>
{
    pub async fn new() -> Arc<Self> {
        let (sender_to_error, receiver_to_error) = MPSC::channel();

        let routs = client::RoutsForClientSide::<WA, RT, MPSC, CH>::new(sender_to_error).await;

        let state = Arc::new(Self {
            id: PhantomData,
            random_number: PhantomData,
            routs: routs,
            jwt: RwLock::new(None),
            is_signed_in: SigBool::default(),
            external_errors: SigExternalError::default(),
            sig_string: PhantomData,
            sig_currency: PhantomData,
            sig_location: PhantomData,
        });

        state.clone().listen_to_error(receiver_to_error);

        state
    }

    pub fn sign_up(
        self: Arc<Self>,
        local_state: Arc<SignUpState<SigString>>,
        feature_state: Arc<AuthFeatureState<SigString, SigBool>>,
    ) {
        RT::spawn_local(async move {
            if feature_state.is_loading.read() == true {
                return;
            }
            feature_state.is_loading.set(true);

            local_state.user_id_error.set(String::new());
            local_state.user_name_error.set(String::new());

            let input = sign_up::Input {
                name: {
                    let name = local_state.user_name.read();
                    match name.is_empty() {
                        true => None,
                        false => Some(name.to_string()),
                    }
                },
                user_id: feature_state.user_id.read().to_string(),
                password: feature_state.user_password.read().to_string(),
            };

            let result = self.routs.clone().sign_up(&input).await;

            match result {
                Ok(Ok(business_output)) => {
                    self.is_signed_in.set(true);
                    *self.jwt.write().unwrap() = Some(business_output.jwt.clone());
                }
                Ok(Err(business_error)) => {
                    local_state.user_id_error.set(match business_error.user_id {
                        Some(_) => String::from("duplicated user"),
                        None => String::new(),
                    });
                    local_state.user_name_error.set(match business_error.name {
                        Some(e) => e,
                        None => String::new(),
                    });
                }
                Err(external_error) => {
                    self.external_errors.set(external_error.to_string());
                }
            }
            feature_state.is_loading.set(false);
        });
    }

    pub fn sign_in(
        self: Arc<Self>,
        local_state: Arc<SignInState<SigString>>,
        feature_state: Arc<AuthFeatureState<SigString, SigBool>>,
    ) {
        RT::spawn_local(async move {
            if feature_state.is_loading.read() == true {
                return;
            }
            feature_state.is_loading.set(true);

            local_state.user_id_error.set(String::new());
            local_state.user_password_error.set(String::new());

            let input = sign_in::Input {
                user_id: feature_state.user_id.read().to_string(),
                password: feature_state.user_password.read().to_string(),
            };

            let result = self.routs.clone().sign_in(&input).await;

            match result {
                Ok(Ok(business_output)) => {
                    self.is_signed_in.set(true);
                    *self.jwt.write().unwrap() = Some(business_output.jwt.clone());
                }
                Ok(Err(business_error)) => {
                    local_state.user_id_error.set(match business_error.user_id {
                        Some(_) => String::from("user not exist"),
                        None => String::new(),
                    });
                    local_state
                        .user_password_error
                        .set(match business_error.password {
                            Some(_) => String::from("wrong password"),
                            None => String::new(),
                        });
                }
                Err(external_error) => {
                    self.external_errors.set(external_error.to_string());
                }
            }

            feature_state.is_loading.set(false);
        });
    }

    fn listen_to_error(self: Arc<Self>, receiver_to_error: MPSC::Receiver<DynamicError>) {
        RT::spawn_local(async move {
            loop {
                let err = receiver_to_error.recv().await.unwrap();
                self.external_errors.set(err.to_string());
            }
        });
    }

    pub async fn create_company(
        self: Arc<Self>,
        local_state: Arc<CreateCompanyState<SigString, SigCurrency>>,
    ) {
        RT::spawn_local(async move {
            let input = create_company::Input {
                jwt: self.jwt.read().unwrap().clone().unwrap_or_default(),
                nonce: Id::generate().to_string(),
                txn_number: RN::generate() as u32,
                company_name: local_state.company_name.read(),
                currency: local_state.currency.read(),
            };

            let result = self.routs.clone().create_company(&input).await;

            match result {
                Ok(Ok(business_output)) => {}
                Ok(Err(business_error)) => {
                    self.external_errors.set(match business_error.nonce {
                        Some(_) => String::from("nonce error"),
                        None => String::new(),
                    });
                    self.is_signed_in.set(business_error.jwt.is_some());
                }
                Err(external_error) => {
                    self.external_errors.set(external_error.to_string());
                }
            }
        });
    }

    pub async fn create_company_branch(
        self: Arc<Self>,
        local_state: CreateCompanyBranchState<SigString, SigCurrency, SigLocation>,
    ) {
        RT::spawn_local(async move {
            let input = create_company_branch::Input {
                jwt: self.jwt.read().unwrap().clone().unwrap_or_default(),
                nonce: Id::generate().to_string(),
                txn_number: RN::generate() as u32,
                company_belong: local_state.company_belong.read(),
                currency: local_state.currency.read(),
                branch_name: local_state.branch_name.read(),
                location: local_state.location.read(),
            };

            let result = self.routs.clone().create_company_branch(&input).await;

            match result {
                Ok(Ok(business_output)) => {}
                Ok(Err(business_error)) => {
                    self.external_errors.set(match business_error.nonce {
                        Some(_) => String::from("nonce error"),
                        None => String::new(),
                    });
                    self.is_signed_in.set(business_error.jwt.is_some());
                    todo!()
                }
                Err(external_error) => {
                    self.external_errors.set(external_error.to_string());
                }
            }
        });
    }
}

pub struct SignInState<SigString>
where
    SigString: Signal<T = String>,
{
    pub user_id_error: SigString,
    pub user_password_error: SigString,
}

pub struct SignUpState<SigString>
where
    SigString: Signal<T = String>,
{
    pub user_name: SigString,
    pub user_id_error: SigString,
    pub user_name_error: SigString,
}

pub struct AuthFeatureState<SigString, SigBool>
where
    SigString: Signal<T = String>,
    SigBool: Signal<T = bool>,
{
    pub user_id: SigString,
    pub user_password: SigString,
    pub is_loading: SigBool,
}

pub struct CreateCompanyState<SigString, SigCurrency>
where
    SigString: Signal<T = String>,
    SigCurrency: Signal<T = db_types::Currency>,
{
    pub company_name: SigString,
    pub currency: SigCurrency,
}

pub struct CreateCompanyBranchState<SigString, SigCurrency, SigLocation>
where
    SigString: Signal<T = String>,
    SigCurrency: Signal<T = db_types::Currency>,
    SigLocation: Signal<T = db_types::Location>,
{
    pub company_belong: SigString,
    pub currency: SigCurrency,
    pub branch_name: SigString,
    pub location: SigLocation,
}
