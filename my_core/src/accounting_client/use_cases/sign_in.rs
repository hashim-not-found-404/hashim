use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::sign_in::Input;
type Type2 = cases::sign_in::Input;
type Type3 = cases::sign_in::MyResult;
type Type4 = Result<SignInOk, cases::sign_in::Error>;

pub(crate) struct SignInOk {
    user_uuid: types::UuidType,
    user_name: String,
}

pub(crate) struct ViewAndCacheType;

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

    fn user_uuid(_: &Self::Type2) -> Option<&types::UuidType> {
        None
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let read_output = LongCache::read(&mut state.cache, &cases::sign_in::ReadInput {
            user_id: data.user_id.clone(),
        })
        .await
        .unwrap();

        if let Some((user_uuid, jwt, user_name)) = read_output.user_rowid_and_password_hash_and_name
        {
            if !jwt.is_empty() {
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
                        user_id:  None,
                        password: Some(cases::sign_in::PasswordError::WrongPassword),
                    });
                }
            }
            None => {
                Err(cases::sign_in::Error {
                    user_id:  Some(cases::sign_in::UserIdError::NotExist),
                    password: None,
                })
            }
        }
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::with_capacity(3);
                let user_uuid = &ok.user_uuid;

                // JWT
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: user_uuid.clone(),
                    resource: resource_utils::Resource::Jwt(ok.jwt.clone()),
                });

                // User ID
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: user_uuid.clone(),
                    resource: resource_utils::Resource::TableUserFieldId(ok.user_id.clone()),
                });

                // User name (optional)
                if let Some(name) = &ok.user_name {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: user_uuid.clone(),
                        resource: resource_utils::Resource::TableUserFieldName(name.clone()),
                    });
                }

                resources
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::SignIn(result) = output {
            match result {
                Ok(ok) => {
                    Ok(SignInOk {
                        user_uuid: ok.user_uuid,
                        user_name: ok.user_name.unwrap_or_default(),
                    })
                }
                Err(err) => Err(err),
            }
        } else {
            unreachable!("{:?}", output)
        }
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        match output {
            Ok(ok) => {
                model.user_name.set(ok.user_name.clone());
            }
            Err(business_error) => {
                model.page_sign_in.user_id_error.set(match business_error.user_id {
                    Some(_) => Some(String::from("user not exist")),
                    None => None,
                });
                model.page_sign_in.user_password_error.set(match business_error.password {
                    Some(_) => Some(String::from("wrong password")),
                    None => None,
                });
            }
        }
    }
}

impl ui_model::SignIn {
    pub(crate) async fn update<
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
            Self::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::SignIn,
                        consent:      i,
                    })
                    .await
                    .unwrap()
            }
            Self::UserId(i) => {
                model.user_id.set(i);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            Self::Password(i) => {
                model.feature_state_auth.user_password.set(i);
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
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.feature_state_auth;
    let local_state = &model.page_sign_in;

    if feature_state.is_loading.read() {
        return;
    }
    feature_state.is_loading.set(true);

    local_state.show_dialog.reset();
    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let user_id = model.user_id.read();
    let input = cases::sign_in::Input {
        user_id:  user_id.clone(),
        password: feature_state.user_password.read(),
    };

    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input);
    let txn_number = Rn::generate();

    {
        let dialog: &'static As::Dialog = &local_state.show_dialog;
        let mut cache = cache;
        let data1 = data.clone();
        let mut cache1 = cache.clone();
        let commander_local_state1 = commander_local_state.clone();
        let mut handle = <Rt>::abortable_spawn_local(async move {
            let mut receiver_to_response = cache1
                .send_to_cache_actor(
                    cache_actor::CachingStrategy::WriteServerOnly,
                    txn_number,
                    data1,
                )
                .await;

            match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => return,
                cache_actor::Response::ServerCannotBeReached => return,
                cache_actor::Response::Data {
                    is_response_from_server,
                    data,
                } => {
                    let result =
                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                    let is_ok = result.is_ok();
                    <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                        &result, model,
                    );

                    if let Ok(ok) = result {
                        model.user_uuid.put(Some(ok.user_uuid));

                        commander_local_state1
                            .sender_to_commander
                            .read()
                            .send(ui_model::Message::CompanyAndBranchSelection(
                                ui_model::CompanyAndBranchSelection::Subscribe,
                            ))
                            .await
                            .unwrap();
                    }

                    commander_local_state1
                        .sender_to_process_manager
                        .read()
                        .send(process_manager::MessageToProcessManager::FromProcess {
                            process_name: process_manager::ProcessName::SignIn,
                            message:      process_manager::MessageFromProcess::Response {
                                is_response_from_server,
                                is_response_ok: is_ok,
                            },
                        })
                        .await
                        .unwrap();
                }
            }
        });
        let (sender_to_process, mut receiver_to_process) = <Mpsc>::channel();
        commander_local_state
            .sender_to_process_manager
            .read()
            .send(process_manager::MessageToProcessManager::FromProcess {
                process_name: process_manager::ProcessName::SignIn,
                message:      process_manager::MessageFromProcess::Subscribe {
                    sender: sender_to_process,
                    dialog: &dialog,
                },
            })
            .await
            .unwrap();
        match receiver_to_process.recv().await.unwrap() {
            process_manager::MessageToProcess::FallBackToCache => {
                let mut receiver_to_response = cache
                    .send_to_cache_actor(
                        cache_actor::CachingStrategy::WriteCacheOnly,
                        txn_number,
                        data,
                    )
                    .await;

                match receiver_to_response.recv().await.unwrap() {
                    cache_actor::Response::CloseTheChannel => return,
                    cache_actor::Response::ServerCannotBeReached => return,
                    cache_actor::Response::Data {
                        is_response_from_server: _,
                        data,
                    } => {
                        let result =
                            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                            &result, model,
                        );

                        if let Ok(ok) = result {
                            model.user_uuid.put(Some(ok.user_uuid));

                            commander_local_state
                                .sender_to_commander
                                .read()
                                .send(ui_model::Message::CompanyAndBranchSelection(
                                    ui_model::CompanyAndBranchSelection::Subscribe,
                                ))
                                .await
                                .unwrap();
                        }
                    }
                }
            }
            process_manager::MessageToProcess::CancelOperation => {}
        }
        handle.abort().await;
    }

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
    let feature_state = &model.feature_state_auth;
    let local_state = &model.page_sign_in;

    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let txn_number = Rn::generate();

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            txn_number,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(cases::sign_in::Input {
                user_id:  model.user_id.read(),
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
    _: Arc<commander::CommanderLocalState<Mpsc, As>>,
    result: Type4,
) {
    <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);
    if let Ok(ok) = result {
        model.user_uuid.put(Some(ok.user_uuid));
    }
}
