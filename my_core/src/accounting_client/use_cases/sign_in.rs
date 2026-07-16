use crate::{
    accounting_client::use_cases::client_domain::{
        cache, cache_actor,
        client_traits::{
            self, CacheAndServerType1, CacheAndServerType2, Mvu, ViewType1, ViewType2,
        },
        commander, process_manager,
        ui_model::{self, HashimSignal},
    },
    accounting_domain::{
        cases::{self},
        request_response, types,
    },
    utility::{
        traits::{self, JoinHandle, Receiver, Sender},
        utils::ReadAndSet,
    },
};
use std::{cmp::Ordering, sync::Arc};

pub(crate) type Type1 = cases::sign_in::Input;
type Type2 = cases::sign_in::Input;
type Type3 = cases::sign_in::MyResult;
pub(crate) struct Type4(pub(crate) Result<SignInOk, cases::sign_in::Error>);

pub(crate) struct SignInOk {
    pub(crate) user_uuid: types::UuidType,
    pub(crate) user_name: String,
}

impl Into<Vec<types::ResourceInfo>> for &cases::sign_in::Ok {
    fn into(self) -> Vec<types::ResourceInfo> {
        use types::{Resource, ResourceInfo};

        let mut resources = Vec::with_capacity(3);
        let user_uuid = &self.user_uuid;

        // JWT
        resources.push(ResourceInfo {
            row_uuid: user_uuid.clone(),
            resource: Resource::Jwt(self.jwt.clone()),
        });

        // User ID
        resources.push(ResourceInfo {
            row_uuid: user_uuid.clone(),
            resource: Resource::TableUserFieldId(self.user_id.clone()),
        });

        // User name (optional)
        if let Some(name) = &self.user_name {
            resources.push(ResourceInfo {
                row_uuid: user_uuid.clone(),
                resource: Resource::TableUserFieldName(name.clone()),
            });
        }

        resources
    }
}

impl ViewType1 for Type1 {
    fn wrap_input(self) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::SignIn(self)
    }
}

impl CacheAndServerType1 for Type2 {
    fn user_uuid(&self) -> Option<&types::UuidType> {
        None
    }

    type Output = Type3;

    async fn state_full_operation<Id: cases::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Self::Output {
        let user_uuid_and_is_jwt_exist = state.cache.read_sign_in(&self.user_id).await;

        if let Some((user_uuid, user_name, is_jwt_exist)) = user_uuid_and_is_jwt_exist {
            if is_jwt_exist {
                return Ok(cases::sign_in::Ok {
                    user_uuid,
                    jwt: types::JsonWebTokenType(String::new()),
                    user_id: self.user_id.clone(),
                    user_name,
                });
            }
        }

        let mut password = None;
        let mut user_uuid = None;
        let mut user_name = None;

        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.id == self.user_id {
                password = Some(user.password.clone());
                user_uuid = Some(rowid);
                user_name = user.name.clone();
            }
        }

        match password {
            Some(password) => {
                if password == self.password {
                    return Ok(self.state_full_operation(
                        &types::JsonWebTokenType(String::new()),
                        &user_uuid.unwrap(),
                        &user_name,
                    ));
                } else {
                    return Err(cases::sign_in::Error {
                        user_id: None,
                        password: Some(cases::sign_in::PasswordError::WrongPassword),
                    });
                }
            }
            None => Err(cases::sign_in::Error {
                user_id: Some(cases::sign_in::UserIdError::NotExist),
                password: None,
            }),
        }
    }
}

impl CacheAndServerType2 for Type3 {
    fn extract_resource(&self) -> Vec<types::ResourceInfo> {
        match self {
            Ok(ok) => ok.into(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> request_response::push_data::OperationsResult {
        request_response::push_data::OperationsResult::SignIn(self)
    }
}

impl ViewType2 for Type4 {
    fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
        if let request_response::push_data::OperationsResult::SignIn(result) = result {
            match result {
                Ok(ok) => Type4(Ok(SignInOk {
                    user_uuid: ok.user_uuid,
                    user_name: ok.user_name.unwrap_or_default(),
                })),
                Err(err) => Type4(Err(err)),
            }
        } else {
            unreachable!("{:?}", result)
        }
    }
}

impl Mvu for ui_model::SignIn {
    async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: cases::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::Type<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                    .await;
            }
            Self::Consent(i) => commander_local_state
                .sender_to_process_manager
                .read()
                .send(process_manager::MessageToProcessManager::FromUser {
                    process_name: process_manager::ProcessName::SignIn,
                    consent: i,
                })
                .await
                .unwrap(),
            Self::UserId(i) => {
                model.page_root.page_auth.auth_feature_state.user_id.set(i);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await;
            }
            Self::Password(i) => {
                model
                    .page_root
                    .page_auth
                    .auth_feature_state
                    .user_password
                    .set(i);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await;
            }
        }
    }
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: cases::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::Type<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.page_root.page_auth.auth_feature_state;
    let local_state = &model.page_root.page_auth.page_sign_in;

    if feature_state.is_loading.read() {
        return;
    }
    feature_state.is_loading.set(true);

    local_state.show_dialog.reset();
    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let user_id = feature_state.user_id.read();
    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::WriteCacheAndServer,
            cases::sign_in::Input {
                user_id: user_id.clone(),
                password: feature_state.user_password.read(),
            }
            .wrap_input(),
        )
        .await;

    let commander_local_state1 = commander_local_state.clone();
    let mut handle = Rt::abortable_spawn_local(async move {
        loop {
            match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => break,
                cache_actor::Response::ServerCannotBeReached => break,
                cache_actor::Response::Data {
                    is_response_from_server,
                    data,
                } => {
                    let is_ok = data.is_ok();

                    if is_response_from_server {
                        commander_local_state1
                            .sender_to_process_manager
                            .read()
                            .send(process_manager::MessageToProcessManager::FromProcess {
                                process_name: process_manager::ProcessName::SignIn,
                                event: process_manager::Event::Completed {
                                    is_response_ok: is_ok,
                                },
                            })
                            .await
                            .unwrap();
                    } else {
                        commander_local_state1
                            .sender_to_process_manager
                            .read()
                            .send(process_manager::MessageToProcessManager::FromProcess {
                                process_name: process_manager::ProcessName::SignIn,
                                event: process_manager::Event::GotResponseFromCache {
                                    is_response_ok: is_ok,
                                },
                            })
                            .await
                            .unwrap();
                    }

                    let result = Type4::unwrap_output(data);
                    handle_apply_result::<Rn, Rt, Id, Mpsc, Rg, As>(
                        &model,
                        commander_local_state1.clone(),
                        result,
                    );
                }
            }
        }
    });

    let (sender_to_process, mut receiver_to_process) = Mpsc::channel();
    commander_local_state
        .sender_to_process_manager
        .read()
        .send(process_manager::MessageToProcessManager::FromProcess {
            process_name: process_manager::ProcessName::SignIn,
            event: process_manager::Event::Subscribe {
                sender: sender_to_process,
                dialog: &local_state.show_dialog,
            },
        })
        .await
        .unwrap();

    match receiver_to_process.recv().await.unwrap() {
        process_manager::ProceedResult::Yes => {
            match commander_local_state.user_uuid.read().clone() {
                Some(_) => {
                    commander_local_state
                        .sender_to_commander
                        .read()
                        .send(ui_model::Message::CompanyAndBranchSelection(
                            ui_model::CompanyAndBranchSelection::Subscribe,
                        ))
                        .await
                        .unwrap();

                    model.page_root.page_after_auth.user_id.set(user_id);
                }
                None => local_state.show_dialog.set(ui_model::Dialog::Error),
            }
        }
        process_manager::ProceedResult::No => {}
    };

    handle.abort().await;
    feature_state.is_loading.reset();
}

async fn handle_check<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: cases::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::Type<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.page_root.page_auth.auth_feature_state;
    let local_state = &model.page_root.page_auth.page_sign_in;

    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            cases::sign_in::Input {
                user_id: feature_state.user_id.read(),
                password: feature_state.user_password.read(),
            }
            .wrap_input(),
        )
        .await;

    match receiver_to_response.recv().await.unwrap() {
        cache_actor::Response::CloseTheChannel => {}
        cache_actor::Response::ServerCannotBeReached => {}
        cache_actor::Response::Data {
            is_response_from_server,
            data,
        } => {
            let result = Type4::unwrap_output(data);
            handle_apply_result::<Rn, Rt, Id, Mpsc, Rg, As>(
                &model,
                commander_local_state.clone(),
                result,
            );
        }
    }
}

fn handle_apply_result<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: cases::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &ui_model::Model<As>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    result: Type4,
) {
    match result.0 {
        Ok(ok) => {
            commander_local_state.user_uuid.put(Some(ok.user_uuid));
            model.page_root.page_after_auth.user_name.set(ok.user_name);
        }
        Err(business_error) => {
            model.page_root.page_auth.page_sign_in.user_id_error.set(
                match business_error.user_id {
                    Some(_) => String::from("user not exist"),
                    None => String::new(),
                },
            );
            model
                .page_root
                .page_auth
                .page_sign_in
                .user_password_error
                .set(match business_error.password {
                    Some(_) => String::from("wrong password"),
                    None => String::new(),
                });
        }
    }
}
