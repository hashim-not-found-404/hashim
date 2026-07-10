use crate::{
    accounting_client::{
        cache_actor, process_manager, ui_model,
        ui_model::HashimSignal,
        use_cases::{self, ViewType1, ViewType2},
    },
    accounting_domain::{
        cases::{self, RowId},
        types,
    },
    mbg,
    utility::{
        traits::{
            self, JoinHandle, MultiProducerSingleConsumer, RandomNumber, Receiver, Runtime, Sender,
        },
        utils::ReadAndSet,
    },
};

use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

pub(crate) struct CommanderLocalState<
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
> {
    pub(crate) sender_to_commander: Mutex<Mpsc::Sender<ui_model::Message>>,
    pub(crate) sender_to_process_manager:
        Mutex<Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>>,
    pub(crate) user_uuid: Mutex<Option<types::UuidType>>,
    pub(crate) selected_company_branch: Mutex<Option<types::UuidType>>,
    pub(crate) aborter_to_company_and_branch_listener: Mutex<Option<Box<dyn FnOnce()>>>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer, As: ui_model::AllSignalTypes>
    CommanderLocalState<Mpsc, As>
{
    pub(crate) fn new(
        sender_to_commander: Mpsc::Sender<ui_model::Message>,
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>,
    ) -> Self {
        CommanderLocalState {
            sender_to_commander: Mutex::new(sender_to_commander),
            sender_to_process_manager: Mutex::new(sender_to_process_manager),
            user_uuid: Mutex::default(),
            selected_company_branch: Mutex::default(),
            aborter_to_company_and_branch_listener: Mutex::default(),
        }
    }
}

pub(crate) trait Mvu {
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
        cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    );
}

pub(crate) mod sign_up {
    use super::*;

    impl Mvu for ui_model::SignUp {
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
            cache: cache_actor::CacheStruct<Mpsc>,
            commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
        ) {
            match self {
                Self::Submit => {
                    handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
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
                    handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await;
                }
                Self::UserId(i) => {
                    model.page_root.page_auth.auth_feature_state.user_id.set(i);
                    handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await;
                }
                Self::Password(i) => {
                    model
                        .page_root
                        .page_auth
                        .auth_feature_state
                        .user_password
                        .set(i);
                    handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await;
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
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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
                    cache_actor::Response::Data(data) => {
                        let is_ok = data.data.is_ok();

                        if data.is_response_from_server {
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

                        let result = use_cases::sign_up::Type4::unwrap_output(data.data);
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
        Id: cases::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        model: &'static ui_model::Model<As>,
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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
            cache_actor::Response::Data(data) => {
                let result = use_cases::sign_up::Type4::unwrap_output(data.data);
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
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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
}

pub(crate) mod sign_in {
    use super::*;

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
            cache: cache_actor::CacheStruct<Mpsc>,
            commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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
                    handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await;
                }
                Self::Password(i) => {
                    model
                        .page_root
                        .page_auth
                        .auth_feature_state
                        .user_password
                        .set(i);
                    handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await;
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
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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
                    cache_actor::Response::Data(data) => {
                        let is_ok = data.data.is_ok();

                        if data.is_response_from_server {
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

                        let result = use_cases::sign_in::Type4::unwrap_output(data.data);
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
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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
            cache_actor::Response::Data(data) => {
                let result = use_cases::sign_in::Type4::unwrap_output(data.data);
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
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
        result: use_cases::sign_in::Type4,
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
}

pub(crate) mod company_and_branch_selection {
    use super::*;

    impl Mvu for ui_model::CompanyAndBranchSelection {
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
            cache: cache_actor::CacheStruct<Mpsc>,
            commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
        ) {
            match self {
                Self::Subscribe => {
                    model
                        .navigator
                        .set(ui_model::Navigator::CompanyBranchSelection(
                            ui_model::CompanyBranchSelection::None,
                        ));

                    handle_list_company_and_branch::<Rn, Rt, Id, Mpsc, Rg, As>(
                        model,
                        cache.clone(),
                        commander_local_state.clone(),
                    )
                    .await;

                    let listener_aborter =
                        handle_list_company_and_branch_listener::<Rn, Rt, Id, Mpsc, Rg, As>(
                            model,
                            cache,
                            commander_local_state.clone(),
                        );

                    *commander_local_state
                        .aborter_to_company_and_branch_listener
                        .lock()
                        .unwrap() = Some(Box::new(listener_aborter));
                }
                Self::UnSubscribe => {
                    let mut guard = commander_local_state
                        .aborter_to_company_and_branch_listener
                        .lock()
                        .unwrap();

                    if let Some(f) = guard.take() {
                        f();
                    }
                }
                Self::ShowCreateCompany => {
                    model
                        .navigator
                        .set(ui_model::Navigator::CompanyBranchSelection(
                            ui_model::CompanyBranchSelection::CreateCompany,
                        ));
                }
                Self::ShowCreateCompanyBranch => {
                    model
                        .navigator
                        .set(ui_model::Navigator::CompanyBranchSelection(
                            ui_model::CompanyBranchSelection::CreateCompanyBranch,
                        ));
                }
                Self::SelectedCompany(i) => {
                    let selected_company = &model
                        .page_root
                        .page_after_auth
                        .page_company_branch_selection
                        .selected_company;

                    match selected_company.read() {
                        Some(old_one) => {
                            if old_one == i {
                                selected_company.set(None)
                            } else {
                                selected_company.set(Some(i))
                            }
                        }
                        None => selected_company.set(Some(i)),
                    }
                }
                Self::SelectedCompanyBranch(i) => {
                    commander_local_state.selected_company_branch.put(Some(i));
                }
            }
        }
    }

    fn handle_list_company_and_branch_listener<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: cases::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        model: &'static ui_model::Model<As>,
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) -> impl FnOnce() {
        let component_id = Rn::generate() as u16;
        let mut cache1 = cache.clone();

        let mut handle = Rt::abortable_spawn_local(async move {
            let mut receiver_to_poke = cache
                .send_subs_to_cache_actor(
                    component_id,
                    use_cases::list_company_and_branch::Type1::subs(),
                )
                .await;

            let data: types::UuidType = commander_local_state.user_uuid.read().clone().unwrap();

            loop {
                receiver_to_poke.recv().await.unwrap();

                let value = cache
                    .send_to_cache_actor(
                        cache_actor::CachingStrategy::ReadCacheOnly,
                        use_cases::list_company_and_branch::Type1 {
                            user_uuid: data.clone(),
                        }
                        .wrap_input(),
                    )
                    .await
                    .recv()
                    .await
                    .unwrap();

                let value = match value {
                    cache_actor::Response::CloseTheChannel => break,
                    cache_actor::Response::ServerCannotBeReached => break,
                    cache_actor::Response::Data(data) => {
                        use_cases::list_company_and_branch::Type4::unwrap_output(data.data)
                    }
                };

                match value.0 {
                    Ok(ok) => model
                        .page_root
                        .page_after_auth
                        .page_company_branch_selection
                        .list
                        .set(ok),
                    Err(_) => {
                        model
                            .navigator
                            .set(ui_model::Navigator::Auth(ui_model::Auth::SignIn));
                        break;
                    }
                };
            }

            cache.send_unsubs_to_cache_actor(component_id).await
        });

        move || {
            Rt::spawn_local(async move {
                handle.abort().await;
                cache1.send_unsubs_to_cache_actor(component_id).await;
            });
        }
    }

    async fn handle_list_company_and_branch<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: cases::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        model: &'static ui_model::Model<As>,
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) {
        let user_uuid = commander_local_state.user_uuid.read().clone().unwrap();

        let mut receiver_to_response = cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::ReadCacheAndServer,
                use_cases::list_company_and_branch::Type1 { user_uuid }.wrap_input(),
            )
            .await;

        loop {
            let value = match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => break,
                cache_actor::Response::ServerCannotBeReached => break,
                cache_actor::Response::Data(data) => {
                    use_cases::list_company_and_branch::Type4::unwrap_output(data.data)
                }
            };

            match value.0 {
                Ok(ok) => model
                    .page_root
                    .page_after_auth
                    .page_company_branch_selection
                    .list
                    .set(ok),
                Err(_) => {
                    model
                        .navigator
                        .set(ui_model::Navigator::Auth(ui_model::Auth::SignIn));
                    break;
                }
            };
        }
    }
}

pub(crate) mod create_company {
    use super::*;

    impl Mvu for ui_model::CreateCompany {
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
            cache: cache_actor::CacheStruct<Mpsc>,
            commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
        ) {
            let page_create_company = &model
                .page_root
                .page_after_auth
                .page_company_branch_selection
                .page_create_company;

            match self {
                Self::Submit => {
                    handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await
                }
                Self::Close => handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model),
                Self::Name(i) => page_create_company.company_name.set(i),
                Self::Currency(i) => page_create_company
                    .currency
                    .set(types::Currency::from_str(i.as_str()).unwrap()),
            }
        }
    }

    fn handle_close<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: cases::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        model: &'static ui_model::Model<As>,
    ) {
        let page_create_company = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company;

        page_create_company.company_name.reset();
        page_create_company.currency.reset();

        model
            .navigator
            .set(ui_model::Navigator::CompanyBranchSelection(
                ui_model::CompanyBranchSelection::None,
            ));
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
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) {
        let data = commander_local_state.user_uuid.read().clone().unwrap();

        let local_state = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company;

        let input = cases::create_company::Input {
            user_uuid: data,
            new_uuid: Id::generate(),
            company_name: local_state.company_name.read(),
            currency: local_state.currency.read(),
        };

        cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::WriteCacheAndServer,
                input.clone().wrap_input(),
            )
            .await;

        handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model);
    }
}

pub(crate) mod create_company_branch {
    use super::*;

    impl Mvu for ui_model::CreateCompanyBranch {
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
            cache: cache_actor::CacheStruct<Mpsc>,
            commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
        ) {
            match self {
                Self::Submit => {
                    handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await
                }
                Self::Consent(i) => {
                    commander_local_state
                        .sender_to_process_manager
                        .read()
                        .send(process_manager::MessageToProcessManager::FromUser {
                            process_name: process_manager::ProcessName::CreateCompanyBranch,
                            consent: i,
                        })
                        .await
                        .unwrap();
                }
                Self::Close => handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model),
                Self::Name(i) => {
                    model
                        .page_root
                        .page_after_auth
                        .page_company_branch_selection
                        .page_create_company_branch
                        .branch_name
                        .set(i);

                    handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state)
                        .await;
                }
                Self::Currency(i) => model
                    .page_root
                    .page_after_auth
                    .page_company_branch_selection
                    .page_create_company_branch
                    .currency
                    .set(types::Currency::from_str(i.as_str()).unwrap()),
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
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company_branch;

        if local_state.is_loading.read() == true {
            return;
        }
        local_state.is_loading.set(true);

        let data = commander_local_state.user_uuid.read().clone().unwrap();

        let input = cases::create_company_branch::Input {
            user_uuid: data,
            new_uuid: Id::generate(),
            company_belong: model
                .page_root
                .page_after_auth
                .page_company_branch_selection
                .selected_company
                .read()
                .unwrap(),
            currency: local_state.currency.read(),
            branch_name: local_state.branch_name.read(),
            location: local_state.location.read(),
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
                    cache_actor::Response::Data(data) => {
                        let is_ok = data.data.is_ok();

                        if data.is_response_from_server {
                            commander_local_state1
                                .sender_to_process_manager
                                .read()
                                .send(process_manager::MessageToProcessManager::FromProcess {
                                    process_name: process_manager::ProcessName::CreateCompanyBranch,
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
                                    process_name: process_manager::ProcessName::CreateCompanyBranch,
                                    event: process_manager::Event::GotResponseFromCache {
                                        is_response_ok: is_ok,
                                    },
                                })
                                .await
                                .unwrap();
                        }

                        let result =
                            use_cases::create_company_branch::Type4::unwrap_output(data.data);

                        match result {
                            Ok(_) => {}
                            Err(business_error) => {
                                mbg!(business_error);
                            }
                        }
                    }
                }
            }
        });

        let (sender_to_process, mut receiver_to_process) = Mpsc::channel();
        commander_local_state
            .sender_to_process_manager
            .read()
            .send(process_manager::MessageToProcessManager::FromProcess {
                process_name: process_manager::ProcessName::CreateCompanyBranch,
                event: process_manager::Event::Subscribe {
                    sender: sender_to_process,
                    dialog: &local_state.show_dialog,
                },
            })
            .await
            .unwrap();

        match receiver_to_process.recv().await.unwrap() {
            process_manager::ProceedResult::Yes => {
                local_state.is_loading.reset();
                handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model);
            }
            process_manager::ProceedResult::No => {}
        };

        handle.abort().await;
        local_state.is_loading.reset();
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
        mut cache: cache_actor::CacheStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company_branch;

        let data = commander_local_state.user_uuid.read().clone().unwrap();

        let input = cases::create_company_branch::Input {
            user_uuid: data,
            new_uuid: Id::generate(),
            company_belong: model
                .page_root
                .page_after_auth
                .page_company_branch_selection
                .selected_company
                .read()
                .unwrap(),
            currency: local_state.currency.read(),
            branch_name: local_state.branch_name.read(),
            location: local_state.location.read(),
        };

        let mut receiver_to_response = cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::ReadCacheOnly,
                input.wrap_input(),
            )
            .await;

        match receiver_to_response.recv().await.unwrap() {
            cache_actor::Response::CloseTheChannel => {}
            cache_actor::Response::ServerCannotBeReached => {}
            cache_actor::Response::Data(data) => {
                let result = use_cases::create_company_branch::Type4::unwrap_output(data.data);

                match result {
                    Ok(_) => {}
                    Err(business_error) => {
                        mbg!(business_error);
                        todo!()
                    }
                }
            }
        }
    }

    fn handle_close<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: cases::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        model: &'static ui_model::Model<As>,
    ) {
        let page_create_company_branch = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company_branch;

        if page_create_company_branch.show_dialog.read() == ui_model::Dialog::Show {
            return;
        }

        if page_create_company_branch.is_loading.read() {
            return;
        }

        page_create_company_branch.branch_name.reset();
        page_create_company_branch.currency.reset();
        page_create_company_branch.location.reset();

        model
            .navigator
            .set(ui_model::Navigator::CompanyBranchSelection(
                ui_model::CompanyBranchSelection::None,
            ));
    }
}
