use crate::prelude::*;

pub trait HashimSignal<T: Default>: Default + Clone {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllSignalTypes: Default {
    type String: HashimSignal<String>;
    type Dialog: HashimSignal<Dialog>;
    type OptionRowId: HashimSignal<Option<db_types::UuidType>>;
    type Bool: HashimSignal<bool>;
    type StringVec: HashimSignal<String>;
    type Currency: HashimSignal<db_types::Currency>;
    type Location: HashimSignal<db_types::Location>;
    type CompanyAndBranchList: HashimSignal<Vec<db_types::Company>>;
}

pub struct State<
    As: AllSignalTypes + 'static,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    ConsentSender: HashimSignal<Option<Mpsc::Sender<IsProceed>>> + 'static,
> {
    _ph: PhantomData<ConsentSender>,
    // here for the app logic
    routs: Arc<web_socket::MyWAMP<At, Mpsc>>,

    // here every field is to display , here is global state
    pub is_signed_in: As::OptionRowId,
    pub selected_company_branch: As::OptionRowId,
    pub external_errors: As::StringVec,
}

impl<
    As: AllSignalTypes,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    ConsentSender: HashimSignal<Option<Mpsc::Sender<IsProceed>>> + 'static,
> Clone for State<As, At, Mpsc, ConsentSender>
{
    fn clone(&self) -> Self {
        Self {
            _ph: self._ph.clone(),
            routs: self.routs.clone(),
            is_signed_in: self.is_signed_in.clone(),
            selected_company_branch: self.selected_company_branch.clone(),
            external_errors: self.external_errors.clone(),
        }
    }
}

impl<
    As: AllSignalTypes,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    ConsentSender: HashimSignal<Option<Mpsc::Sender<IsProceed>>> + 'static,
> State<As, At, Mpsc, ConsentSender>
{
    pub fn new() -> Self {
        let (sender_to_error, receiver_to_error) = Mpsc::channel();

        let external_errors = As::StringVec::default();
        Self::listen_to_error_actor(receiver_to_error, external_errors.clone());

        let routs = Arc::new(web_socket::MyWAMP::<At, Mpsc>::new(sender_to_error.clone()));
        let routs1 = routs.clone();
        At::Rt::spawn_local(async move {
            let url = format!("ws://{}/ws", ADDRESS);
            routs1.connect_to_url(&url).await;
        });

        let state = Self {
            _ph: PhantomData,
            routs,
            is_signed_in: As::OptionRowId::default(),
            selected_company_branch: As::OptionRowId::default(),
            external_errors,
        };

        state
    }

    fn listen_to_error_actor(
        mut receiver_to_error: Mpsc::Receiver<HashimError>,
        external_errors_signal: As::StringVec,
    ) {
        At::Rt::spawn_local(async move {
            loop {
                let err = receiver_to_error.recv().await.unwrap();
                external_errors_signal.set(err.to_string());
            }
        });
    }

    fn consent_receiver_and_dialog_actors(
        sender_to_consent_from_dialog: ConsentSender,
        is_user_want_to_proceed: Arc<Mutex<IsProceed>>,
        mut sender: Mpsc::Sender<()>,
    ) -> <At::Rt as Runtime>::JoinHandel<()> {
        At::Rt::abortable_spawn_local(async move {
            let (sender_to_consent, mut receiver_to_consent) = Mpsc::channel();
            sender_to_consent_from_dialog.set(Some(sender_to_consent));
            *is_user_want_to_proceed.lock().unwrap() = receiver_to_consent.recv().await.unwrap();

            sender.send(()).await.unwrap();
        })
    }

    fn response_receiver_actor(
        input: push_data::OperationsInput,
        strategy: web_socket::CachingStrategy,
        mut sender: Mpsc::Sender<()>,
        response: Arc<Mutex<Option<web_socket::Data>>>,
        routs: Arc<web_socket::MyWAMP<At, Mpsc>>,
    ) -> <<At as AllClientTypes>::Rt as Runtime>::JoinHandel<()> {
        At::Rt::abortable_spawn_local(async move {
            let mut receiver_to_response = routs.send_to_cache_actor(strategy, input).await;

            loop {
                let result = receiver_to_response.recv().await.unwrap();
                match result {
                    web_socket::Response::CloseTheChannel => {
                        break;
                    }
                    web_socket::Response::ServerCannotBeReached => {
                        break;
                    }
                    web_socket::Response::Data(data) => {
                        *response.lock().unwrap() = Some(data);
                        sender.send(()).await.unwrap();
                    }
                }
            }
        })
    }

    pub fn sign_up(
        self,
        sender_to_consent_from_dialog: ConsentSender,
        is_submit: bool,
        local_state: SignUpState<As>,
        feature_state: AuthFeatureState<As>,
    ) {
        At::Rt::spawn_local(async move {
            if feature_state.is_loading.read() == true {
                return;
            }
            feature_state.is_loading.set(true);

            local_state.show_dialog.reset();
            local_state.user_id_error.reset();
            local_state.user_name_error.reset();

            let new_uuid = At::Id::generate().to_uuid();
            let input = sign_up::Input {
                new_uuid: new_uuid.clone(),
                name: {
                    let name = local_state.user_name.read();
                    match name.is_empty() {
                        true => None,
                        false => Some(name.to_string()),
                    }
                },
                user_id: feature_state.user_id.read(),
                password: feature_state.user_password.read(),
            };

            let strategy = if is_submit {
                web_socket::CachingStrategy::WriteCacheAndServer
            } else {
                web_socket::CachingStrategy::ReadCacheOnly
            };

            let (sender, mut receiver) = Mpsc::channel();
            let is_user_want_to_proceed = Arc::new(Mutex::new(IsProceed::Wait));
            let response = Arc::new(Mutex::new(None));

            let mut handel_consent = Self::consent_receiver_and_dialog_actors(
                sender_to_consent_from_dialog,
                is_user_want_to_proceed.clone(),
                sender.clone(),
            );

            let mut handel_response = Self::response_receiver_actor(
                input.wrap_input(),
                strategy,
                sender,
                response.clone(),
                self.routs.clone(),
            );

            loop {
                match receiver.recv().await {
                    Ok(_) => {}
                    Err(_) => break,
                }

                if let Some(response) = response.lock().unwrap().clone() {
                    let result = sign_up::Result::unwrap_output(response.data);

                    let is_ok = result.is_ok();
                    match result {
                        Ok(_) => {}
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
                        match is_proceed(
                            is_ok,
                            response.is_response_from_server,
                            *is_user_want_to_proceed.lock().unwrap(),
                        ) {
                            IsProceed::Wait => {
                                local_state.show_dialog.set(Dialog::Show);
                                continue;
                            }
                            IsProceed::Yes => {
                                self.is_signed_in.set(Some(new_uuid));
                                local_state.show_dialog.reset();
                                break;
                            }
                            IsProceed::No => {
                                local_state.show_dialog.reset();
                                break;
                            }
                        };
                    } else {
                        break;
                    }
                }
            }

            handel_consent.abort().await;
            handel_response.abort().await;
            feature_state.is_loading.reset();
        });
    }

    pub fn sign_in(
        self,
        sender_to_consent_from_dialog: ConsentSender,
        is_submit: bool,
        local_state: SignInState<As>,
        feature_state: AuthFeatureState<As>,
    ) {
        At::Rt::spawn_local(async move {
            if feature_state.is_loading.read() == true {
                return;
            }
            feature_state.is_loading.set(true);

            local_state.show_dialog.reset();
            local_state.user_id_error.reset();
            local_state.user_password_error.reset();

            let user_id = feature_state.user_id.read();
            let input = sign_in::Input {
                user_id: user_id.clone(),
                password: feature_state.user_password.read(),
            };

            let strategy = if is_submit {
                web_socket::CachingStrategy::WriteCacheAndServer
            } else {
                web_socket::CachingStrategy::ReadCacheOnly
            };

            let (sender, mut receiver) = Mpsc::channel();
            let is_user_want_to_proceed = Arc::new(Mutex::new(IsProceed::Wait));
            let response = Arc::new(Mutex::new(None));

            let mut handel_consent = Self::consent_receiver_and_dialog_actors(
                sender_to_consent_from_dialog,
                is_user_want_to_proceed.clone(),
                sender.clone(),
            );

            let mut handel_response = Self::response_receiver_actor(
                input.wrap_input(),
                strategy,
                sender,
                response.clone(),
                self.routs.clone(),
            );

            let mut user_uuid = None;
            loop {
                match receiver.recv().await {
                    Ok(_) => {}
                    Err(_) => break,
                }

                if let Some(response) = response.lock().unwrap().clone() {
                    let result = operations::SignInResultForView::unwrap_output(response.data);

                    let is_ok = result.0.is_ok();
                    match result.0 {
                        Ok(ok) => {
                            user_uuid = Some(ok);
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

                    if is_submit {
                        match is_proceed(
                            is_ok,
                            response.is_response_from_server,
                            *is_user_want_to_proceed.lock().unwrap(),
                        ) {
                            IsProceed::Wait => {
                                local_state.show_dialog.set(Dialog::Show);
                                continue;
                            }
                            IsProceed::Yes => {
                                match user_uuid {
                                    Some(_) => local_state.show_dialog.reset(),
                                    None => local_state.show_dialog.set(Dialog::Error),
                                }
                                self.is_signed_in.set(user_uuid);
                                break;
                            }
                            IsProceed::No => {
                                local_state.show_dialog.reset();
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }

            handel_consent.abort().await;
            handel_response.abort().await;
            feature_state.is_loading.reset();
        });
    }

    pub fn list_company_and_branch(self, local_state: As::CompanyAndBranchList) {
        At::Rt::spawn_local(async move {
            let data = self.is_signed_in.read().unwrap();
            let mut receiver_to_response = self
                .routs
                .send_to_cache_actor(
                    web_socket::CachingStrategy::ReadCacheAndServer,
                    list_company_and_branch::Input { user_uuid: data }.wrap_input(),
                )
                .await;

            loop {
                let value = match receiver_to_response.recv().await.unwrap() {
                    web_socket::Response::CloseTheChannel => break,
                    web_socket::Response::ServerCannotBeReached => break,
                    web_socket::Response::Data(data) => {
                        operations::ListCompanyAndBranchForView::unwrap_output(data.data)
                    }
                };

                let value = match value.0 {
                    Ok(ok) => ok,
                    Err(_) => {
                        self.is_signed_in.set(None);
                        break;
                    }
                };

                local_state.set(value);
            }
        });
    }

    pub fn list_company_and_branch_listener(
        self,
        local_state: As::CompanyAndBranchList,
    ) -> impl FnOnce() {
        let component_id: u16 = At::Rn::generate() as u16;
        let routs = self.routs.clone();

        let mut handel = At::Rt::abortable_spawn_local(async move {
            let mut receiver_to_poke = self
                .routs
                .send_subs_to_cache_actor(component_id, list_company_and_branch::Input::subs())
                .await;

            loop {
                receiver_to_poke.recv().await.unwrap();

                let data = self.is_signed_in.read().unwrap();
                let value = self
                    .routs
                    .send_to_cache_actor(
                        web_socket::CachingStrategy::ReadCacheOnly,
                        list_company_and_branch::Input { user_uuid: data }.wrap_input(),
                    )
                    .await
                    .recv()
                    .await
                    .unwrap();

                let value = match value {
                    web_socket::Response::CloseTheChannel => break,
                    web_socket::Response::ServerCannotBeReached => break,
                    web_socket::Response::Data(data) => {
                        operations::ListCompanyAndBranchForView::unwrap_output(data.data)
                    }
                };

                let value = match value.0 {
                    Ok(ok) => ok,
                    Err(_) => {
                        self.is_signed_in.set(None);
                        break;
                    }
                };

                local_state.set(value);
            }

            self.routs.send_unsubs_to_cache_actor(component_id).await
        });

        move || {
            At::Rt::spawn_local(async move {
                handel.abort().await;
                routs.send_unsubs_to_cache_actor(component_id).await;
            });
        }
    }

    pub fn create_company(self, local_state: CreateCompanyState<As>) {
        At::Rt::spawn_local(async move {
            let input = create_company::Input {
                user_uuid: self.is_signed_in.read().unwrap(),
                new_uuid: At::Id::generate().to_uuid(),
                company_name: local_state.company_name.read(),
                currency: local_state.currency.read(),
            };

            self.routs
                .send_to_cache_actor(
                    web_socket::CachingStrategy::WriteCacheAndServer,
                    input.clone().wrap_input(),
                )
                .await;

            local_state.company_name.reset();
            local_state.currency.reset();
        });
    }

    pub fn create_company_branch(self, is_submit: bool, local_state: CreateCompanyBranchState<As>) {
        At::Rt::spawn_local(async move {
            todo!();
            // let input = create_company_branch::Input {
            //     user_uuid: self.is_signed_in.read().unwrap(),
            //     new_uuid: At::Id::generate().to_uuid(),
            //     company_belong: local_state.company_belong.read(),
            //     currency: local_state.currency.read(),
            //     branch_name: local_state.branch_name.read(),
            //     location: local_state.location.read(),
            // };
        });
    }
}

#[derive(Default, Clone, PartialEq)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}

#[derive(Default, Clone)]
pub struct SignInState<As: AllSignalTypes> {
    pub show_dialog: As::Dialog,
    pub user_id_error: As::String,
    pub user_password_error: As::String,
}

#[derive(Default, Clone)]
pub struct SignUpState<As: AllSignalTypes> {
    pub show_dialog: As::Dialog,
    pub user_name: As::String,
    pub user_id_error: As::String,
    pub user_name_error: As::String,
}

#[derive(Default, Clone)]
pub struct AuthFeatureState<As: AllSignalTypes> {
    pub user_id: As::String,
    pub user_password: As::String,
    pub is_loading: As::Bool,
}

#[derive(Default, Clone)]
pub struct CreateCompanyState<As: AllSignalTypes> {
    pub company_name: As::String,
    pub currency: As::Currency,
}

#[derive(Default, Clone)]
pub struct CreateCompanyBranchState<As: AllSignalTypes> {
    pub company_belong: As::String,
    pub currency: As::Currency,
    pub branch_name: As::String,
    pub location: As::Location,
}

#[derive(Debug, Clone, Copy)]
pub enum IsProceed {
    Wait,
    Yes,
    No,
}

fn is_proceed(
    is_ok: bool,
    is_response_from_server: bool,
    is_user_want_to_proceed: IsProceed,
) -> IsProceed {
    match (is_response_from_server, is_ok, is_user_want_to_proceed) {
        (true, true, IsProceed::Wait) => IsProceed::Yes,
        (true, true, IsProceed::Yes) => IsProceed::Yes,
        (true, true, IsProceed::No) => IsProceed::Yes,
        (true, false, IsProceed::Wait) => IsProceed::No,
        (true, false, IsProceed::Yes) => IsProceed::No,
        (true, false, IsProceed::No) => IsProceed::No,
        (false, true, IsProceed::Wait) => IsProceed::Wait,
        (false, true, IsProceed::Yes) => IsProceed::Yes,
        (false, true, IsProceed::No) => IsProceed::No,
        (false, false, IsProceed::Wait) => IsProceed::Wait,
        (false, false, IsProceed::Yes) => IsProceed::Yes,
        (false, false, IsProceed::No) => IsProceed::No,
    }
}
