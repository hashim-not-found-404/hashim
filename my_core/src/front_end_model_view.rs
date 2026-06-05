use crate::prelude::*;

pub trait Signal<T>: Default {
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllSignalTypes: Default {
    type SigString: Signal<String>;
    type SigBool: Signal<bool>;
    type SigExternalError: Signal<String>;
    type SigCurrency: Signal<db_types::Currency>;
    type SigLocation: Signal<db_types::Location>;
}

pub struct State<
    At: AllClientTypes + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    As: AllSignalTypes + 'static,
> {
    // here for the app logic
    routs: web_socket::MyWAMP<At, MPSC>,

    // here every field is to display , here is global state
    pub is_signed_in: As::SigBool,
    pub external_errors: As::SigExternalError,
}

impl<
    At: AllClientTypes + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
    // signals
    As: AllSignalTypes,
> State<At, MPSC, As>
{
    pub fn new() -> Arc<Self> {
        let (sender_to_error, receiver_to_error) = MPSC::channel();

        let state = Arc::new(Self {
            routs: web_socket::MyWAMP::<At, MPSC>::new(sender_to_error.clone()),
            is_signed_in: As::SigBool::default(),
            external_errors: As::SigExternalError::default(),
        });

        let state1 = state.clone();
        At::Rt::spawn_local(async move {
            let url = format!("ws://{}/ws", ADDRESS);
            state1.routs.connect_to_url(&url).await;
        });

        state.clone().listen_to_error(receiver_to_error);

        state
    }

    pub fn sign_up(
        self: Arc<Self>, // TODO pass reciever
        is_submit: bool,
        local_state: Arc<SignUpState<As>>,
        feature_state: Arc<AuthFeatureState<As>>,
    ) {
        At::Rt::spawn_local(async move {
            if feature_state.is_loading.read() == true {
                return;
            }
            feature_state.is_loading.set(true);

            local_state.user_id_error.set(String::new());
            local_state.user_name_error.set(String::new());

            let input = sign_up::Input {
                new_uuid: At::Id::generate().to_string(),
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

            let (sender, receiver) = MPSC::channel();
            self.routs
                .send_to_cache_actor(web_socket::Query {
                    is_submit,
                    sender: sender,
                    data: input.clone().map_input(),
                })
                .await;

            if is_submit {
                let self1 = self.clone();
                let local_state1 = local_state.clone();
                At::Rt::spawn_local(async move {
                    loop {
                        if self1.routs.is_online() {
                            At::Rt::sleep(Duration::from_secs(10)).await;
                        } else {
                            At::Rt::sleep(Duration::from_secs(1)).await;
                        }

                        local_state1.show_dialog.set(true);
                    }
                });
            }

            let mut response = None;
            let mut is_user_want_to_proceed = false;
            loop {
                // todo!("i need to add timeout and ask to proceed offline");
                let result = receiver.recv().await.unwrap();
                response = match result {
                    Some(result) => Some(result),
                    None => break,
                };

                if let Some(response) = response {
                    let result = sign_up::Input::unwrap(response.data);

                    let is_ok = result.is_ok();
                    match result {
                        Ok(business_output) => {}
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

                    if is_submit {
                        if is_proceed(
                            is_ok,
                            self.routs.is_online(),
                            response.is_response_from_server,
                            is_user_want_to_proceed,
                        ) {
                            self.is_signed_in.set(true);
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }

            feature_state.is_loading.set(false);
        });
    }

    pub fn sign_in(
        self: Arc<Self>,
        is_submit: bool,
        local_state: Arc<SignInState<As>>,
        feature_state: Arc<AuthFeatureState<As>>,
    ) {
        At::Rt::spawn_local(async move {
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

            let (sender, receiver) = MPSC::channel();
            self.routs
                .send_to_cache_actor(web_socket::Query {
                    is_submit,
                    sender: sender,
                    data: input.clone().map_input(),
                })
                .await;

            let mut is_user_want_to_proceed = false;
            loop {
                let result = receiver.recv().await.unwrap();
                let response = match result {
                    Some(result) => result,
                    None => break,
                };
                let result = sign_in::Input::unwrap(response.data);

                let is_ok = result.is_ok();
                match result {
                    Ok(business_output) => {}
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

                if is_submit {
                    if is_proceed(
                        is_ok,
                        self.routs.is_online(),
                        response.is_response_from_server,
                        is_user_want_to_proceed,
                    ) {
                        self.is_signed_in.set(true);
                        break;
                    }
                } else {
                    break;
                }
            }

            feature_state.is_loading.set(false);
        });
    }

    fn listen_to_error(self: Arc<Self>, receiver_to_error: MPSC::Receiver<DynamicError>) {
        At::Rt::spawn_local(async move {
            loop {
                let err = receiver_to_error.recv().await.unwrap();
                self.external_errors.set(err.to_string());
            }
        });
    }

    pub fn create_company(
        self: Arc<Self>,
        is_submit: bool,
        local_state: Arc<CreateCompanyState<As>>,
    ) {
        At::Rt::spawn_local(async move {
            let input = create_company::Input {
                user_uuid: String::new(),
                new_uuid: At::Id::generate().to_string(),
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

    pub fn create_company_branch(
        self: Arc<Self>,
        is_submit: bool,
        local_state: Arc<CreateCompanyBranchState<As>>,
    ) {
        At::Rt::spawn_local(async move {
            let input = create_company_branch::Input {
                user_uuid: String::new(),
                new_uuid: At::Id::generate().to_string(),
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

#[derive(Default)]
pub struct SignInState<As: AllSignalTypes> {
    pub user_id_error: As::SigString,
    pub user_password_error: As::SigString,
}

#[derive(Default)]
pub struct SignUpState<As: AllSignalTypes> {
    pub show_dialog: As::SigBool,
    pub user_name: As::SigString,
    pub user_id_error: As::SigString,
    pub user_name_error: As::SigString,
}

#[derive(Default)]
pub struct AuthFeatureState<As: AllSignalTypes> {
    pub user_id: As::SigString,
    pub user_password: As::SigString,
    pub is_loading: As::SigBool,
}

#[derive(Default)]
pub struct CreateCompanyState<As: AllSignalTypes> {
    pub company_name: As::SigString,
    pub currency: As::SigCurrency,
}

#[derive(Default)]
pub struct CreateCompanyBranchState<As: AllSignalTypes> {
    pub company_belong: As::SigString,
    pub currency: As::SigCurrency,
    pub branch_name: As::SigString,
    pub location: As::SigLocation,
}

fn is_proceed(
    is_ok: bool,
    is_online: bool,
    is_response_from_server: bool,
    is_user_want_to_proceed: bool,
) -> bool {
    match (
        is_ok,
        is_online,
        is_response_from_server,
        is_user_want_to_proceed,
    ) {
        (true, true, true, true) => true,
        (true, true, true, false) => true,
        (true, true, false, true) => true,
        (true, true, false, false) => false,
        (true, false, true, true) => true,
        (true, false, true, false) => true,
        (true, false, false, true) => true,
        (true, false, false, false) => false,
        (false, true, true, true) => false,
        (false, true, true, false) => false,
        (false, true, false, true) => true,
        (false, true, false, false) => false,
        (false, false, true, true) => false,
        (false, false, true, false) => false,
        (false, false, false, true) => true,
        (false, false, false, false) => false,
    }
}
