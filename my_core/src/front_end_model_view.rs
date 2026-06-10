use crate::{
    cache_query_operations::{GetUserUuidInput, QueryOperations},
    prelude::*,
};

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
}

pub struct State<
    As: AllSignalTypes + 'static,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    ConsentSender: HashimSignal<Option<Mpsc::Sender<()>>> + 'static,
> {
    _ph: PhantomData<ConsentSender>,
    // here for the app logic
    routs: Arc<web_socket::MyWAMP<At, Mpsc>>,

    // here every field is to display , here is global state
    pub is_signed_in: As::OptionRowId,
    pub external_errors: As::StringVec,
}

impl<
    As: AllSignalTypes,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    ConsentSender: HashimSignal<Option<Mpsc::Sender<()>>> + 'static,
> Clone for State<As, At, Mpsc, ConsentSender>
{
    fn clone(&self) -> Self {
        Self {
            _ph: self._ph.clone(),
            routs: self.routs.clone(),
            is_signed_in: self.is_signed_in.clone(),
            external_errors: self.external_errors.clone(),
        }
    }
}

impl<
    As: AllSignalTypes,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
    ConsentSender: HashimSignal<Option<Mpsc::Sender<()>>> + 'static,
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
            external_errors,
        };

        state
    }

    fn listen_to_error_actor(
        mut receiver_to_error: Mpsc::Receiver<DynamicError>,
        external_errors_signal: As::StringVec,
    ) {
        At::Rt::spawn_local(async move {
            loop {
                let err = receiver_to_error.recv().await.unwrap();
                external_errors_signal.set(err.to_string());
            }
        });
    }

    fn timeout_dialog_actor(
        routs: Arc<web_socket::MyWAMP<At, Mpsc>>,
        is_submit: bool,
        show_dialog: As::Dialog,
    ) -> <<At as AllClientTypes>::Rt as Runtime>::JoinHandel<()> {
        At::Rt::abortable_spawn_local(async move {
            if is_submit {
                loop {
                    if routs.is_online() {
                        At::Rt::sleep(Duration::from_secs(3)).await;
                    } else {
                        At::Rt::sleep(Duration::from_secs(1)).await;
                    }

                    show_dialog.set(Dialog::Show);
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

            let mut receiver_to_response = self
                .routs
                .send_to_cache_actor(is_submit, input.clone().map_input())
                .await;

            let mut handel = Self::timeout_dialog_actor(
                self.routs.clone(),
                is_submit,
                local_state.show_dialog.clone(),
            );

            let (sender_to_consent, mut receiver_to_consent) = Mpsc::channel();
            sender_to_consent_from_dialog.set(Some(sender_to_consent));
            let mut is_user_want_to_proceed = false;
            let mut response = None;
            loop {
                match At::Rt::select(receiver_to_consent.recv(), receiver_to_response.recv()).await
                {
                    Either::One(_) => {
                        is_user_want_to_proceed = true;
                        handel.abort().await;
                    }
                    Either::Two(result) => {
                        response = match result.unwrap() {
                            Some(result) => Some(result),
                            None => break,
                        }
                    }
                };

                if let Some(response) = response.clone() {
                    let result = sign_up::Input::unwrap(response.data);

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
                        if is_proceed(
                            is_ok,
                            self.routs.is_online(),
                            response.is_response_from_server,
                            is_user_want_to_proceed,
                        ) {
                            self.is_signed_in.set(Some(new_uuid));
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            handel.abort().await;
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

            let mut receiver_to_response = self
                .routs
                .send_to_cache_actor(is_submit, input.clone().map_input())
                .await;

            let mut handel = Self::timeout_dialog_actor(
                self.routs.clone(),
                is_submit,
                local_state.show_dialog.clone(),
            );

            let (sender_to_consent, mut receiver_to_consent) = Mpsc::channel();
            sender_to_consent_from_dialog.set(Some(sender_to_consent));
            let mut is_user_want_to_proceed = false;
            let mut response = None;
            loop {
                match At::Rt::select(receiver_to_consent.recv(), receiver_to_response.recv()).await
                {
                    Either::One(_) => {
                        is_user_want_to_proceed = true;
                        handel.abort().await;
                    }
                    Either::Two(result) => {
                        response = match result.unwrap() {
                            Some(result) => Some(result),
                            None => break,
                        }
                    }
                };

                if let Some(response) = response.clone() {
                    let result = sign_in::Input::unwrap(response.data);

                    let is_ok = result.is_ok();
                    match result {
                        Ok(_) => {}
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
                            handel.abort().await;
                            At::Rt::sleep(Duration::from_secs(3)).await;
                            let result = self
                                .routs
                                .send_query_to_cache_actor(
                                    cache_query_operations::CacheQueryInput::GetUserUuid(
                                        GetUserUuidInput {
                                            user_id: user_id.clone(),
                                        },
                                    ),
                                )
                                .await;

                            let result = cache_query_operations::GetUserUuidInput::unwrap(result);

                            if result.is_none() {
                                local_state.show_dialog.set(Dialog::Error);
                            }

                            self.is_signed_in.set(result);
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            handel.abort().await;
            feature_state.is_loading.reset();
        });
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
                .send_to_cache_actor(true, input.clone().map_input())
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
