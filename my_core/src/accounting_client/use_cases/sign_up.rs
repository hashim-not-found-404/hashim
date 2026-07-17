use std::sync::Arc;

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
        cases::{
            self,
            utility::types::{self, MyErrorTrait},
        },
        request_response,
    },
    utility::{
        traits::{self, JoinHandle, Receiver, Sender},
        utils::ReadAndSet,
    },
};

pub(crate) type Type1 = cases::sign_up::Input;
type Type2 = cases::sign_up::Input;
type Type3 = cases::sign_up::MyResult;
pub(crate) type Type4 = cases::sign_up::MyResult;

impl Into<Vec<types::ResourceInfo>> for &cases::sign_up::Ok {
    fn into(self) -> Vec<types::ResourceInfo> {
        let mut resource = Vec::with_capacity(3);

        resource.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::Jwt(self.jwt.clone()),
        });

        resource.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableUserFieldId(self.user_id.clone()),
        });

        if let Some(user_name) = &self.user_name {
            resource.push(types::ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: types::Resource::TableUserFieldName(user_name.clone()),
            });
        }

        resource
    }
}

impl ViewType1 for Type1 {
    fn wrap_input(self) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::SignUp(self)
    }
}

impl CacheAndServerType1 for Type2 {
    fn user_uuid(&self) -> Option<&types::UuidType> {
        Some(&self.new_uuid)
    }

    type Output = Type3;
    async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Self::Output {
        let (is_new_uuid_exist, is_user_id_exist) =
            state.read_sign_up(&self.new_uuid, &self.user_id).await;
        let errr = self.state_full_check::<Id>(is_new_uuid_exist, is_user_id_exist);
        if errr.is_there_error() {
            return Err(errr);
        }

        let result = cases::sign_up::Ok {
            new_uuid: self.new_uuid.clone(),
            user_id: self.user_id.clone(),
            user_name: self.name.clone(),
            hashed_password: String::new(),
            jwt: types::JsonWebTokenType(String::new()),
        };

        return Ok(result);
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
        request_response::push_data::OperationsResult::SignUp(self)
    }
}

impl ViewType2 for Type4 {
    fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
        if let request_response::push_data::OperationsResult::SignUp(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl Mvu for ui_model::SignUp {
    async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await
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
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await;
            }
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
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
            input.wrap_input(),
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
                    let result = Type4::unwrap_output(data);
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
            cases::sign_up::Input {
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
                Some(_) => String::from("duplicated user"),
                None => String::new(),
            });
            local_state.user_name_error.set(match business_error.name {
                Some(e) => e,
                None => String::new(),
            });
        }
    }
}
