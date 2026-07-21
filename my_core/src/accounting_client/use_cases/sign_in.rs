use crate::{
    accounting_client::use_cases::client_domain::{
        cache, cache_actor,
        client_traits::{self, ViewAndCache},
        commander, process_manager,
        ui_model::{self, HashimSignal},
    },
    accounting_domain::{
        cases::{
            self,
            utility::{resource_utils, types},
        },
        request_response,
    },
    utility::{
        traits::{self, JoinHandle, Receiver, Sender},
        utils::ReadAndSet,
    },
};
use std::{marker::PhantomData, sync::Arc};

pub(crate) type Type1 = cases::sign_in::Input;
type Type2 = cases::sign_in::Input;
type Type3 = cases::sign_in::MyResult;
pub(crate) struct Type4(pub(crate) Result<SignInOk, cases::sign_in::Error>);

pub(crate) struct SignInOk {
    pub(crate) user_uuid: types::UuidType,
    pub(crate) user_name: String,
}

impl Into<Vec<resource_utils::ResourceInfo>> for &cases::sign_in::Ok {
    fn into(self) -> Vec<resource_utils::ResourceInfo> {
        use resource_utils::{Resource, ResourceInfo};

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

struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::SignIn(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        None
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let user_uuid_and_is_jwt_exist = state.cache.read_sign_in(&data.user_id).await;

        if let Some((user_uuid, user_name, is_jwt_exist)) = user_uuid_and_is_jwt_exist {
            if is_jwt_exist {
                return Ok(cases::sign_in::Ok {
                    user_uuid,
                    jwt: types::JsonWebTokenType(String::new()),
                    user_id: data.user_id.clone(),
                    user_name,
                });
            }
        }

        let mut password = None;
        let mut user_uuid = None;
        let mut user_name = None;

        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.id == data.user_id {
                password = Some(user.password.clone());
                user_uuid = Some(rowid);
                user_name = user.name.clone();
            }
        }

        match password {
            Some(password) => {
                if password == data.password {
                    return Ok(data.state_full_operation(
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

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => ok.into(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(data: Self::Type3) -> request_response::push_data::OperationsResult {
        request_response::push_data::OperationsResult::SignIn(data)
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::SignIn(result) = output {
            match result {
                Ok(ok) => Type4(Ok(SignInOk {
                    user_uuid: ok.user_uuid,
                    user_name: ok.user_name.unwrap_or_default(),
                })),
                Err(err) => Type4(Err(err)),
            }
        } else {
            unreachable!("{:?}", output)
        }
    }
}

impl ui_model::SignIn {
    async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
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
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            Self::Password(i) => {
                model
                    .page_root
                    .page_auth
                    .auth_feature_state
                    .user_password
                    .set(i);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
        }
    }
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
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
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(cases::sign_in::Input {
                user_id: user_id.clone(),
                password: feature_state.user_password.read(),
            }),
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
                    let result: Type4 =
                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                    let is_ok = result.0.is_ok();

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

                    handle_apply_result::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.page_root.page_auth.auth_feature_state;
    let local_state = &model.page_root.page_auth.page_sign_in;

    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(cases::sign_in::Input {
                user_id: feature_state.user_id.read(),
                password: feature_state.user_password.read(),
            }),
        )
        .await;

    match receiver_to_response.recv().await.unwrap() {
        cache_actor::Response::CloseTheChannel => {}
        cache_actor::Response::ServerCannotBeReached => {}
        cache_actor::Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result: Type4 =
                <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
            handle_apply_result::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
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
                    Some(_) => Some(String::from("user not exist")),
                    None => None,
                },
            );
            model
                .page_root
                .page_auth
                .page_sign_in
                .user_password_error
                .set(match business_error.password {
                    Some(_) => Some(String::from("wrong password")),
                    None => None,
                });
        }
    }
}
