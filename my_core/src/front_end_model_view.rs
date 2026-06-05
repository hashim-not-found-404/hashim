use crate::prelude::*;

pub trait Signal<T>: Default {
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllSignalTypes: Default {
    type String: Signal<String>;
    type Bool: Signal<bool>;
    type StringVec: Signal<String>;
    type Currency: Signal<db_types::Currency>;
    type Location: Signal<db_types::Location>;
}

pub struct State<
    As: AllSignalTypes + 'static,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
> {
    // here for the app logic
    routs: web_socket::MyWAMP<At, Mpsc>,

    // here every field is to display , here is global state
    pub is_signed_in: As::Bool,
    pub external_errors: As::StringVec,
}

impl<As: AllSignalTypes, At: AllClientTypes + 'static, Mpsc: MultiProducerSingleConsumer + 'static>
    State<As, At, Mpsc>
{
    pub fn new() -> Arc<Self> {
        let (sender_to_error, receiver_to_error) = Mpsc::channel();

        let state = Arc::new(Self {
            routs: web_socket::MyWAMP::<At, Mpsc>::new(sender_to_error.clone()),
            is_signed_in: As::Bool::default(),
            external_errors: As::StringVec::default(),
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
                new_uuid: At::Id::generate().to_row_id(),
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

            let (sender, receiver) = Mpsc::channel();
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
                user_id: feature_state.user_id.read().into(),
                password: feature_state.user_password.read().to_string(),
            };

            let (sender, receiver) = Mpsc::channel();
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

    fn listen_to_error(self: Arc<Self>, receiver_to_error: Mpsc::Receiver<DynamicError>) {
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
                user_uuid: todo!(),
                new_uuid: At::Id::generate().to_row_id(),
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
                user_uuid: todo!(),
                new_uuid: At::Id::generate().to_row_id(),
                company_belong: todo!("local_state.company_belong.read()"),
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
    pub user_id_error: As::String,
    pub user_password_error: As::String,
}

#[derive(Default)]
pub struct SignUpState<As: AllSignalTypes> {
    pub show_dialog: As::Bool,
    pub user_name: As::String,
    pub user_id_error: As::String,
    pub user_name_error: As::String,
}

#[derive(Default)]
pub struct AuthFeatureState<As: AllSignalTypes> {
    pub user_id: As::String,
    pub user_password: As::String,
    pub is_loading: As::Bool,
}

#[derive(Default)]
pub struct CreateCompanyState<As: AllSignalTypes> {
    pub company_name: As::String,
    pub currency: As::Currency,
}

#[derive(Default)]
pub struct CreateCompanyBranchState<As: AllSignalTypes> {
    pub company_belong: As::String,
    pub currency: As::Currency,
    pub branch_name: As::String,
    pub location: As::Location,
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
