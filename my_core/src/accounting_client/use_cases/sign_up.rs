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
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::marker::PhantomData;
use std::sync::Arc;

type Type1 = cases::sign_up::Input;
type Type2 = cases::sign_up::Input;
type Type3 = cases::sign_up::MyResult;
type Type4 = cases::sign_up::MyResult;

impl Into<Vec<resource_utils::ResourceInfo>> for &cases::sign_up::Ok {
    fn into(self) -> Vec<resource_utils::ResourceInfo> {
        let mut resource = Vec::with_capacity(3);

        resource.push(resource_utils::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: resource_utils::Resource::Jwt(self.jwt.clone()),
        });

        resource.push(resource_utils::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: resource_utils::Resource::TableUserFieldId(self.user_id.clone()),
        });

        if let Some(user_name) = &self.user_name {
            resource.push(resource_utils::ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: resource_utils::Resource::TableUserFieldName(user_name.clone()),
            });
        }

        resource
    }
}

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::sign_up::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::sign_up::ReadInput,
    ) -> Result<cases::sign_up::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await.unwrap();

        for (uuid, user) in &db.state_of_pending_txn.user {
            if user.id == read_input.user_id {
                read_output.is_user_id_exist = true;
            }
            if *uuid == read_input.new_uuid {
                read_output.is_new_uuid_exist = true;
            }
        }

        Ok(read_output)
    }
}

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::SignUp(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.new_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let errr = data
            .state_full_check::<Id, Cache<Ch, LongCache>>(state)
            .await
            .unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        let result = cases::sign_up::Ok {
            new_uuid: data.new_uuid.clone(),
            user_id: data.user_id.clone(),
            user_name: data.name.clone(),
            hashed_password: String::new(),
            jwt: types::JsonWebTokenType(String::new()),
        };

        return Ok(result);
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => ok.into(),
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::SignUp(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }
}

impl ui_model::SignUp {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>,
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
                .await
            }
            Self::Consent(i) => commander_local_state
                .sender_to_process_manager
                .read()
                .send(process_manager::MessageToProcessManager::FromUser {
                    process_name: process_manager::ProcessName::SignUp,
                    consent: i,
                })
                .await
                .unwrap(),
            Self::UserName(i) => {
                model.page_root.page_auth.page_sign_up.user_name.set(i);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
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
    LongCache: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.page_root.page_auth.auth_feature_state;
    let local_state = &model.page_root.page_auth.page_sign_up;

    if feature_state.is_loading.read() == true {
        return;
    }
    feature_state.is_loading.set(true);

    local_state.show_dialog.reset();
    local_state.user_id_error.reset();
    local_state.user_name_error.reset();

    let new_uuid = Id::generate();
    let input = cases::sign_up::Input {
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

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::WriteCacheAndServer,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input),
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
                    let is_ok = result.is_ok();

                    if is_response_from_server {
                        commander_local_state1
                            .sender_to_process_manager
                            .read()
                            .send(process_manager::MessageToProcessManager::FromProcess {
                                process_name: process_manager::ProcessName::SignUp,
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
                                process_name: process_manager::ProcessName::SignUp,
                                event: process_manager::Event::GotResponseFromCache {
                                    is_response_ok: is_ok,
                                },
                            })
                            .await
                            .unwrap();
                    }

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
            process_name: process_manager::ProcessName::SignUp,
            event: process_manager::Event::Subscribe {
                sender: sender_to_process,
                dialog: &local_state.show_dialog,
            },
        })
        .await
        .unwrap();

    match receiver_to_process.recv().await.unwrap() {
        process_manager::ProceedResult::Yes => {
            model
                .page_root
                .page_after_auth
                .user_id
                .set(feature_state.user_id.read());

            model
                .page_root
                .page_after_auth
                .user_name
                .set(local_state.user_name.read());

            commander_local_state.user_uuid.put(Some(new_uuid));

            commander_local_state
                .sender_to_commander
                .read()
                .send(ui_model::Message::CompanyAndBranchSelection(
                    ui_model::CompanyAndBranchSelection::Subscribe,
                ))
                .await
                .unwrap();
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
    LongCache: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.page_root.page_auth.auth_feature_state;
    let local_state = &model.page_root.page_auth.page_sign_up;

    local_state.user_id_error.reset();
    local_state.user_name_error.reset();

    let new_uuid = Id::generate();
    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(cases::sign_up::Input {
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &ui_model::Model<As>,
    _: Arc<commander::CommanderLocalState<Mpsc, As>>,
    result: cases::sign_up::MyResult,
) {
    let local_state = &model.page_root.page_auth.page_sign_up;
    match result {
        Ok(_) => {}
        Err(business_error) => {
            local_state.user_id_error.set(match business_error.user_id {
                Some(_) => Some(String::from("duplicated user")),
                None => None,
            });
            local_state.user_name_error.set(match business_error.name {
                Some(e) => Some(e),
                None => None,
            });
        }
    }
}
