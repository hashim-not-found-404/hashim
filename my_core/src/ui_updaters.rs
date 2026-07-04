use crate::{
    cache_actor, db_types, decider, mbg,
    operations::{self, ViewType1, ViewType2},
    process_manager,
    traits::{
        AllClientTypes, JoinHandle, MultiProducerSingleConsumer, RandomNumber, Receiver, RowId,
        Runtime, Sender,
    },
    ui_effect,
    ui_model::{self, HashimSignal},
};
use std::{str::FromStr, sync::Arc};

pub(crate) trait Mvu {
    async fn update<At: AllClientTypes>(
        self,
        model: &'static ui_model::Model<At>,
        cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    );
}

pub mod sign_up {
    use super::*;

    #[derive(Debug)]
    pub enum Msg {
        Submit,
        Consent(process_manager::UserConsent),
        UserName(String),
        UserId(String),
        Password(String),
    }

    impl Mvu for Msg {
        async fn update<At: AllClientTypes>(
            self,
            model: &'static ui_model::Model<At>,
            cache: cache_actor::CacheStruct<At>,
            commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        ) {
            match self {
                Msg::Submit => handle_submit(model, cache, commander_local_state).await,
                Msg::Consent(i) => commander_local_state
                    .sender_to_process_manager
                    .lock()
                    .unwrap()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::SignUp,
                        consent: i,
                    })
                    .await
                    .unwrap(),
                Msg::UserName(i) => {
                    model.page_root.page_auth.page_sign_up.user_name.set(i);
                    handle_check(model, cache, commander_local_state).await;
                }
                Msg::UserId(i) => {
                    model.page_root.page_auth.auth_feature_state.user_id.set(i);
                    handle_check(model, cache, commander_local_state).await;
                }
                Msg::Password(i) => {
                    model
                        .page_root
                        .page_auth
                        .auth_feature_state
                        .user_password
                        .set(i);
                    handle_check(model, cache, commander_local_state).await;
                }
            }
        }
    }

    async fn handle_submit<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
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

        let new_uuid = At::Id::generate();
        let input = decider::sign_up::Input {
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
        let mut handle = At::Rt::abortable_spawn_local(async move {
            loop {
                match receiver_to_response.recv().await.unwrap() {
                    cache_actor::Response::CloseTheChannel => break,
                    cache_actor::Response::ServerCannotBeReached => break,
                    cache_actor::Response::Data(data) => {
                        let is_ok = data.data.is_ok();

                        if data.is_response_from_server {
                            commander_local_state1
                                .sender_to_process_manager
                                .lock()
                                .unwrap()
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
                                .lock()
                                .unwrap()
                                .send(process_manager::MessageToProcessManager::FromProcess {
                                    process_name: process_manager::ProcessName::SignUp,
                                    event: process_manager::Event::GotResponseFromCache {
                                        is_response_ok: is_ok,
                                    },
                                })
                                .await
                                .unwrap();
                        }

                        let result = operations::sign_up::Type4::unwrap_output(data.data);
                        handle_apply_result(&model, commander_local_state1.clone(), result);
                    }
                }
            }
        });

        let (sender_to_process, mut receiver_to_process) = At::Mpsc::channel();
        commander_local_state
            .sender_to_process_manager
            .lock()
            .unwrap()
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

                *commander_local_state.user_uuid.lock().unwrap() = Some(new_uuid);

                commander_local_state
                    .sender_to_commander
                    .lock()
                    .unwrap()
                    .send(ui_model::Message::CompanyAndBranchSelection(
                        company_and_branch_selection::Msg::Subscribe,
                    ))
                    .await
                    .unwrap();
            }
            process_manager::ProceedResult::No => {}
        };

        handle.abort().await;
        feature_state.is_loading.reset();
    }

    async fn handle_check<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    ) {
        let feature_state = &model.page_root.page_auth.auth_feature_state;
        let local_state = &model.page_root.page_auth.page_sign_up;

        local_state.user_id_error.reset();
        local_state.user_name_error.reset();

        let new_uuid = At::Id::generate();
        let mut receiver_to_response = cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::ReadCacheOnly,
                decider::sign_up::Input {
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
                let result = operations::sign_up::Type4::unwrap_output(data.data);
                handle_apply_result(&model, commander_local_state.clone(), result);
            }
        }
    }

    fn handle_apply_result<At: AllClientTypes>(
        model: &ui_model::Model<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        result: decider::sign_up::Result,
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

pub mod sign_in {
    use super::*;

    #[derive(Debug)]
    pub enum Msg {
        Submit,
        Consent(process_manager::UserConsent),
        UserId(String),
        Password(String),
    }

    impl Mvu for Msg {
        async fn update<At: AllClientTypes>(
            self,
            model: &'static ui_model::Model<At>,
            cache: cache_actor::CacheStruct<At>,
            commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        ) {
            match self {
                Msg::Submit => {
                    handle_submit(model, cache, commander_local_state).await;
                }
                Msg::Consent(i) => commander_local_state
                    .sender_to_process_manager
                    .lock()
                    .unwrap()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::SignIn,
                        consent: i,
                    })
                    .await
                    .unwrap(),
                Msg::UserId(i) => {
                    model.page_root.page_auth.auth_feature_state.user_id.set(i);
                    handle_check(model, cache, commander_local_state).await;
                }
                Msg::Password(i) => {
                    model
                        .page_root
                        .page_auth
                        .auth_feature_state
                        .user_password
                        .set(i);
                    handle_check(model, cache, commander_local_state).await;
                }
            }
        }
    }

    async fn handle_submit<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
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
                decider::sign_in::Input {
                    user_id: user_id.clone(),
                    password: feature_state.user_password.read(),
                }
                .wrap_input(),
            )
            .await;

        let commander_local_state1 = commander_local_state.clone();
        let mut handle = At::Rt::abortable_spawn_local(async move {
            loop {
                match receiver_to_response.recv().await.unwrap() {
                    cache_actor::Response::CloseTheChannel => break,
                    cache_actor::Response::ServerCannotBeReached => break,
                    cache_actor::Response::Data(data) => {
                        let is_ok = data.data.is_ok();

                        if data.is_response_from_server {
                            commander_local_state1
                                .sender_to_process_manager
                                .lock()
                                .unwrap()
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
                                .lock()
                                .unwrap()
                                .send(process_manager::MessageToProcessManager::FromProcess {
                                    process_name: process_manager::ProcessName::SignIn,
                                    event: process_manager::Event::GotResponseFromCache {
                                        is_response_ok: is_ok,
                                    },
                                })
                                .await
                                .unwrap();
                        }

                        let result = operations::sign_in::Type4::unwrap_output(data.data);
                        handle_apply_result(&model, commander_local_state1.clone(), result);
                    }
                }
            }
        });

        let (sender_to_process, mut receiver_to_process) = At::Mpsc::channel();
        commander_local_state
            .sender_to_process_manager
            .lock()
            .unwrap()
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
                match commander_local_state.user_uuid.lock().unwrap().clone() {
                    Some(_) => {
                        commander_local_state
                            .sender_to_commander
                            .lock()
                            .unwrap()
                            .send(ui_model::Message::CompanyAndBranchSelection(
                                company_and_branch_selection::Msg::Subscribe,
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

    async fn handle_check<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    ) {
        let feature_state = &model.page_root.page_auth.auth_feature_state;
        let local_state = &model.page_root.page_auth.page_sign_in;

        local_state.user_id_error.reset();
        local_state.user_password_error.reset();

        let mut receiver_to_response = cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::ReadCacheOnly,
                decider::sign_in::Input {
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
                let result = operations::sign_in::Type4::unwrap_output(data.data);
                handle_apply_result(&model, commander_local_state.clone(), result);
            }
        }
    }

    fn handle_apply_result<At: AllClientTypes>(
        model: &ui_model::Model<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        result: operations::sign_in::Type4,
    ) {
        match result.0 {
            Ok(ok) => {
                *commander_local_state.user_uuid.lock().unwrap() = Some(ok.user_uuid);
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

pub mod company_and_branch_selection {
    use super::*;

    #[derive(Debug)]
    pub enum Msg {
        Subscribe,
        UnSubscribe,
        ShowCreateCompany,
        ShowCreateCompanyBranch,
        SelectedCompany(db_types::UuidType),
        SelectedCompanyBranch(db_types::UuidType),
    }

    impl Mvu for Msg {
        async fn update<At: AllClientTypes>(
            self,
            model: &'static ui_model::Model<At>,
            cache: cache_actor::CacheStruct<At>,
            commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        ) {
            match self {
                Msg::Subscribe => {
                    model
                        .navigator
                        .set(ui_model::Navigator::CompanyBranchSelection(
                            ui_model::CompanyBranchSelection::None,
                        ));

                    handle_list_company_and_branch(
                        model,
                        cache.clone(),
                        commander_local_state.clone(),
                    )
                    .await;

                    let listener_aborter = handle_list_company_and_branch_listener(
                        model,
                        cache,
                        commander_local_state.clone(),
                    );

                    *commander_local_state
                        .aborter_to_company_and_branch_listener
                        .lock()
                        .unwrap() = Some(Box::new(listener_aborter));
                }
                Msg::UnSubscribe => {
                    let mut guard = commander_local_state
                        .aborter_to_company_and_branch_listener
                        .lock()
                        .unwrap();

                    if let Some(f) = guard.take() {
                        f();
                    }
                }
                Msg::ShowCreateCompany => {
                    model
                        .navigator
                        .set(ui_model::Navigator::CompanyBranchSelection(
                            ui_model::CompanyBranchSelection::CreateCompany,
                        ));
                }
                Msg::ShowCreateCompanyBranch => {
                    model
                        .navigator
                        .set(ui_model::Navigator::CompanyBranchSelection(
                            ui_model::CompanyBranchSelection::CreateCompanyBranch,
                        ));
                }
                Msg::SelectedCompany(i) => {
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
                Msg::SelectedCompanyBranch(i) => {
                    *commander_local_state
                        .selected_company_branch
                        .lock()
                        .unwrap() = Some(i);
                }
            }
        }
    }

    fn handle_list_company_and_branch_listener<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    ) -> impl FnOnce() {
        let component_id = At::Rn::generate() as u16;
        let mut cache1 = cache.clone();

        let mut handle = At::Rt::abortable_spawn_local(async move {
            let mut receiver_to_poke = cache
                .send_subs_to_cache_actor(
                    component_id,
                    operations::list_company_and_branch::Type1::subs(),
                )
                .await;

            let data: db_types::UuidType = commander_local_state
                .user_uuid
                .lock()
                .unwrap()
                .clone()
                .unwrap();

            loop {
                receiver_to_poke.recv().await.unwrap();

                let value = cache
                    .send_to_cache_actor(
                        cache_actor::CachingStrategy::ReadCacheOnly,
                        operations::list_company_and_branch::Type1 {
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
                        operations::list_company_and_branch::Type4::unwrap_output(data.data)
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
            At::Rt::spawn_local(async move {
                handle.abort().await;
                cache1.send_unsubs_to_cache_actor(component_id).await;
            });
        }
    }

    async fn handle_list_company_and_branch<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    ) {
        let user_uuid = commander_local_state
            .user_uuid
            .lock()
            .unwrap()
            .clone()
            .unwrap();

        let mut receiver_to_response = cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::ReadCacheAndServer,
                operations::list_company_and_branch::Type1 { user_uuid }.wrap_input(),
            )
            .await;

        loop {
            let value = match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => break,
                cache_actor::Response::ServerCannotBeReached => break,
                cache_actor::Response::Data(data) => {
                    operations::list_company_and_branch::Type4::unwrap_output(data.data)
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

pub mod create_company {
    use super::*;

    #[derive(Debug)]
    pub enum Msg {
        Submit,
        Close,
        Name(String),
        Currency(String),
    }

    impl Mvu for Msg {
        async fn update<At: AllClientTypes>(
            self,
            model: &'static ui_model::Model<At>,
            cache: cache_actor::CacheStruct<At>,
            commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        ) {
            let page_create_company = &model
                .page_root
                .page_after_auth
                .page_company_branch_selection
                .page_create_company;

            match self {
                Msg::Submit => handle_submit(model, cache, commander_local_state).await,
                Msg::Close => handle_close(model),
                Msg::Name(i) => page_create_company.company_name.set(i),
                Msg::Currency(i) => page_create_company
                    .currency
                    .set(db_types::Currency::from_str(i.as_str()).unwrap()),
            }
        }
    }

    fn handle_close<At: AllClientTypes>(model: &'static ui_model::Model<At>) {
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

    async fn handle_submit<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    ) {
        let data = commander_local_state
            .user_uuid
            .lock()
            .unwrap()
            .clone()
            .unwrap();

        let local_state = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company;

        let input = decider::create_company::Input {
            user_uuid: data,
            new_uuid: At::Id::generate(),
            company_name: local_state.company_name.read(),
            currency: local_state.currency.read(),
        };

        cache
            .send_to_cache_actor(
                cache_actor::CachingStrategy::WriteCacheAndServer,
                input.clone().wrap_input(),
            )
            .await;

        handle_close(model);
    }
}

pub mod create_company_branch {
    use super::*;

    #[derive(Debug)]
    pub enum Msg {
        Submit,
        Consent(process_manager::UserConsent),
        Close,
        Name(String),
        Currency(String),
    }

    impl Mvu for Msg {
        async fn update<At: AllClientTypes>(
            self,
            model: &'static ui_model::Model<At>,
            cache: cache_actor::CacheStruct<At>,
            commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
        ) {
            match self {
                Msg::Submit => handle_submit(model, cache, commander_local_state).await,
                Msg::Consent(i) => {
                    commander_local_state
                        .sender_to_process_manager
                        .lock()
                        .unwrap()
                        .send(process_manager::MessageToProcessManager::FromUser {
                            process_name: process_manager::ProcessName::CreateCompanyBranch,
                            consent: i,
                        })
                        .await
                        .unwrap();
                }
                Msg::Close => handle_close(model),
                Msg::Name(i) => {
                    model
                        .page_root
                        .page_after_auth
                        .page_company_branch_selection
                        .page_create_company_branch
                        .branch_name
                        .set(i);

                    handle_check(model, cache, commander_local_state).await;
                }
                Msg::Currency(i) => model
                    .page_root
                    .page_after_auth
                    .page_company_branch_selection
                    .page_create_company_branch
                    .currency
                    .set(db_types::Currency::from_str(i.as_str()).unwrap()),
            }
        }
    }

    async fn handle_submit<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
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

        let data = commander_local_state
            .user_uuid
            .lock()
            .unwrap()
            .clone()
            .unwrap();

        let input = decider::create_company_branch::Input {
            user_uuid: data,
            new_uuid: At::Id::generate(),
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
        let mut handle = At::Rt::abortable_spawn_local(async move {
            loop {
                match receiver_to_response.recv().await.unwrap() {
                    cache_actor::Response::CloseTheChannel => break,
                    cache_actor::Response::ServerCannotBeReached => break,
                    cache_actor::Response::Data(data) => {
                        let is_ok = data.data.is_ok();

                        if data.is_response_from_server {
                            commander_local_state1
                                .sender_to_process_manager
                                .lock()
                                .unwrap()
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
                                .lock()
                                .unwrap()
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
                            operations::create_company_branch::Type4::unwrap_output(data.data);

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

        let (sender_to_process, mut receiver_to_process) = At::Mpsc::channel();
        commander_local_state
            .sender_to_process_manager
            .lock()
            .unwrap()
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
                handle_close(model);
            }
            process_manager::ProceedResult::No => {}
        };

        handle.abort().await;
        local_state.is_loading.reset();
    }

    async fn handle_check<At: AllClientTypes>(
        model: &'static ui_model::Model<At>,
        mut cache: cache_actor::CacheStruct<At>,
        commander_local_state: Arc<ui_effect::CommanderLocalState<At>>,
    ) {
        let local_state = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company_branch;

        let data = commander_local_state
            .user_uuid
            .lock()
            .unwrap()
            .clone()
            .unwrap();

        let input = decider::create_company_branch::Input {
            user_uuid: data,
            new_uuid: At::Id::generate(),
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
                let result = operations::create_company_branch::Type4::unwrap_output(data.data);

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

    fn handle_close<At: AllClientTypes>(model: &'static ui_model::Model<At>) {
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
