use crate::prelude::*;

pub trait Signal {
    type T;
    fn read(&self) -> Self::T;
    fn set(&self, v: Self::T);
}

pub struct State<
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    CH: CacheIO + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    Id: RowId + 'static,
    SigString: Signal<T = String> + 'static,
    SigBool: Signal<T = bool> + 'static,
    SigExternalError: Signal<T = String> + 'static,
    SigCurrency: Signal<T = db_types::Currency> + 'static,
    SigLocation: Signal<T = db_types::Location> + 'static,
> {
    // here for the app logic
    _ph: PhantomData<(Id, RN, SigString, SigCurrency, SigLocation)>,
    routs: client::RoutsForClientSide<WS, DE, RN, RT, CH, Id, MPSC>,

    // here every field to display
    // here is global state
    pub is_signed_in: SigBool,
    // here is feature state
    pub external_errors: SigExternalError,
}

impl<
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    CH: CacheIO + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    // signals
    Id: RowId + 'static,
    SigString: Signal<T = String>,
    SigBool: Signal<T = bool> + Default,
    SigExternalError: Signal<T = String> + Default,
    SigCurrency: Signal<T = db_types::Currency>,
    SigLocation: Signal<T = db_types::Location>,
>
    State<
        RN,
        WS,
        DE,
        RT,
        CH,
        MPSC,
        Id,
        SigString,
        SigBool,
        SigExternalError,
        SigCurrency,
        SigLocation,
    >
{
    pub async fn new() -> Arc<Self> {
        let (sender_to_error, receiver_to_error) = MPSC::channel();

        let routs =
            client::RoutsForClientSide::<WS, DE, RN, RT, CH, Id, MPSC>::new(sender_to_error).await;

        let state = Arc::new(Self {
            _ph: PhantomData,
            routs: routs,
            is_signed_in: SigBool::default(),
            external_errors: SigExternalError::default(),
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
                new_uuid: Id::generate().to_string(),
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

            let result = self.routs.sign_up(&input).await;

            match result {
                Ok(business_output) => {
                    self.is_signed_in.set(true);
                }
                Err(business_error) => {
                    local_state.user_id_error.set(match business_error.user_id {
                        Some(_) => String::from("duplicated user"),
                        None => String::new(),
                    });
                    local_state.user_name_error.set(match business_error.name {
                        Some(e) => e,
                        None => String::new(),
                    });
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

            let result = self.routs.sign_in(&input).await;

            match result {
                Ok(business_output) => {
                    self.is_signed_in.set(true);
                }
                Err(business_error) => {
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
                user_uuid: String::new(),
                new_uuid: Id::generate().to_string(),
                company_name: local_state.company_name.read(),
                currency: local_state.currency.read(),
            };

            let txn = push_data::OperationsInput::CreateCompany(input);

            // let result = self.routs.cache.write_txn(&txn).await;

            // match result {
            //     Ok(_) => {}
            //     Err(external_error) => {
            //         self.external_errors.set(external_error.to_string());
            //     }
            // }
        });
    }

    pub async fn create_company_branch(
        self: Arc<Self>,
        local_state: Arc<CreateCompanyBranchState<SigString, SigCurrency, SigLocation>>,
    ) {
        RT::spawn_local(async move {
            let input = create_company_branch::Input {
                user_uuid: String::new(),
                new_uuid: Id::generate().to_string(),
                company_belong: local_state.company_belong.read(),
                currency: local_state.currency.read(),
                branch_name: local_state.branch_name.read(),
                location: local_state.location.read(),
            };

            // TODO : make offline check

            let txn = push_data::OperationsInput::CreateCompanyBranch(input);

            // let result = self.routs.cache.write_txn(&txn).await;

            // match result {
            //     Ok(_) => {}
            //     Err(external_error) => {
            //         self.external_errors.set(external_error.to_string());
            //     }
            // }
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
