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
            utility::{
                resource_utils,
                types::{self, MyErrorTrait},
            },
        },
        request_response,
    },
    mbg,
    utility::{
        traits::{self, JoinHandle, Receiver, Sender},
        utils::ReadAndSet,
    },
};
use std::{str::FromStr, sync::Arc};

pub(crate) type Type1 = cases::create_company_branch::Input;
type Type2 = cases::create_company_branch::Input;
type Type3 = cases::create_company_branch::MyResult;
pub(crate) type Type4 = cases::create_company_branch::MyResult;

impl Into<Vec<resource_utils::ResourceInfo>> for &cases::create_company_branch::Ok {
    fn into(self) -> Vec<resource_utils::ResourceInfo> {
        let branch_uuid = self.new_uuid.clone();

        vec![
            // Branch fields
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid.clone(),
                resource: resource_utils::Resource::TableCompanyBranchFieldName(
                    self.branch_name.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid.clone(),
                resource: resource_utils::Resource::TableCompanyBranchFieldCompanyBelong(
                    self.company_belong.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid.clone(),
                resource: resource_utils::Resource::TableCompanyBranchFieldLocation(
                    self.location.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid.clone(),
                resource: resource_utils::Resource::TableCompanyBranchFieldCurrency(
                    self.currency.clone(),
                ),
            },
            // Access control for this branch (row_uuid is the branch UUID)
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid.clone(),
                resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldRole(
                    self.role.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid.clone(),
                resource: resource_utils::Resource::TableAccessControlForCompanyBranchFieldUser(
                    self.user_uuid.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: branch_uuid,
                resource:
                    resource_utils::Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                        self.new_uuid.clone(),
                    ),
            },
        ]
    }
}

impl ViewType1 for Type1 {
    fn wrap_input(self) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateCompanyBranch(self)
    }
}

impl CacheAndServerType1 for Type2 {
    fn user_uuid(&self) -> Option<&types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = Type3;

    async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Self::Output {
        let (user_roles, is_company_belong_exist, is_branch_name_used) = state
            .read_create_company_branch(&self.user_uuid, &self.company_belong, &self.branch_name)
            .await;

        let errr = self.state_full_check::<Id>(
            &user_roles,
            false,
            is_company_belong_exist,
            is_branch_name_used,
        );
        if errr.is_there_error() {
            return Err(errr);
        }

        let result = self.state_less_operation();

        return Ok(result);
    }
}

impl CacheAndServerType2 for Type3 {
    fn extract_resource(&self) -> Vec<resource_utils::ResourceInfo> {
        match self {
            Ok(ok) => ok.into(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(self) -> request_response::push_data::OperationsResult {
        request_response::push_data::OperationsResult::CreateCompanyBranch(self)
    }
}

impl ViewType2 for Type4 {
    fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
        if let request_response::push_data::OperationsResult::CreateCompanyBranch(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl Mvu for ui_model::CreateCompanyBranch {
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

                handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await;
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
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
        cache_actor::Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result = Type4::unwrap_output(data);

            match result {
                Ok(_) => {}
                Err(business_error) => {
                    local_state.branch_name_error.set(todo!());
                    local_state.location_error.set(todo!());
                }
            }
        }
    }
}

fn handle_close<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
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
