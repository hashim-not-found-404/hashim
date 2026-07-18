use crate::{
    accounting_client::use_cases::client_domain::{
        cache, cache_actor,
        client_traits::{
            self, CacheAndServerType1, CacheAndServerType2, Mvu, ViewType1, ViewType2,
        },
        commander, process_manager,
        ui_model::{self, HashimSignal, PageCreateAccount},
    },
    accounting_domain::{
        cases::{
            self,
            utility::types::{self, MyErrorTrait},
        },
        request_response,
    },
    mbg,
    utility::{
        traits::{self, JoinHandle, Receiver, Sender},
        utils::ReadAndSet,
    },
};
use std::sync::Arc;

pub(crate) type Type1 = cases::create_account::Input;
type Type2 = cases::create_account::Input;
type Type3 = cases::create_account::MyResult;
pub(crate) type Type4 = cases::create_account::MyResult;

impl Into<Vec<types::ResourceInfo>> for &cases::create_account::Ok {
    fn into(self) -> Vec<types::ResourceInfo> {
        let mut resources = Vec::new();

        resources.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableAccountFieldCompanyBelong(
                self.belong_to_company.clone(),
            ),
        });

        resources.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableAccountFieldIsDebit(self.is_debit.clone()),
        });

        resources.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableAccountFieldIsPermanentAccount(
                self.is_permanent_account.clone(),
            ),
        });

        resources.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableAccountFieldName(self.account_name.clone()),
        });

        resources.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableAccountFieldNotes(self.notes.clone()),
        });

        resources.push(types::ResourceInfo {
            row_uuid: self.new_uuid.clone(),
            resource: types::Resource::TableAccountFieldUnitOfMeasurementOfQuantity(
                self.unit_of_measurement_of_quantity.clone(),
            ),
        });

        resources
    }
}

impl ViewType1 for Type1 {
    fn wrap_input(self) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateAccount(self)
    }
}

impl CacheAndServerType1 for Type2 {
    fn user_uuid(&self) -> Option<&types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = Type3;

    async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
        &self,
        cache: &mut cache::State<Ch>,
    ) -> Self::Output {
        let errr = self.state_less_check::<Id>();

        if errr.is_there_error() {
            return Err(errr);
        }

        let read_output = cache
            .read_create_account(&cases::create_account::ReadInput {
                user_uuid: self.user_uuid.clone(),
                new_uuid: self.new_uuid.clone(),
                belong_to_company: self.belong_to_company.clone(),
                account_name: self.account_name.clone(),
            })
            .await;

        let errr = self.state_full_check(&read_output);

        if errr.is_there_error() {
            return Err(errr);
        }

        let ok = self.state_less_operation();

        return Ok(ok);
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
        request_response::push_data::OperationsResult::CreateAccount(self)
    }
}

impl ViewType2 for Type4 {
    fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
        if let request_response::push_data::OperationsResult::CreateAccount(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl Mvu for ui_model::CreateAccount {
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
        let local_state = &model
            .page_root
            .page_after_auth
            .page_home
            .page_create_account;

        match self {
            ui_model::CreateAccount::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await
            }
            Self::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::CreateAccount,
                        consent: i,
                    })
                    .await
                    .unwrap();
            }
            ui_model::CreateAccount::Clean => handle_clean::<Rn, Rt, Id, Mpsc, Rg, As>(model),
            ui_model::CreateAccount::IsDebit(v) => local_state.is_debit.set(v),
            ui_model::CreateAccount::IsPermanentAccount(v) => {
                local_state.is_permanent_account.set(v)
            }
            ui_model::CreateAccount::AccountName(v) => {
                local_state.account_name.set(v);
                handle_check::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await;
            }
            ui_model::CreateAccount::Notes(v) => local_state.notes.set(v),
            ui_model::CreateAccount::UnitOfMeasurementOfQuantity(v) => {
                local_state.unit_of_measurement_of_quantity.set(v)
            }
        }
    }
}

fn handle_clean<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
) {
    let local_state = &model
        .page_root
        .page_after_auth
        .page_home
        .page_create_account;

    local_state.account_name.reset();
    local_state.is_debit.reset();
    local_state.is_permanent_account.reset();
    local_state.notes.reset();
    local_state.unit_of_measurement_of_quantity.reset();
    local_state.is_loading.reset();
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
    let user_uuid = commander_local_state.user_uuid.read().clone().unwrap();

    let local_state = &model
        .page_root
        .page_after_auth
        .page_home
        .page_create_account;

    let input = cases::create_account::Input {
        user_uuid,
        new_uuid: Id::generate(),
        is_debit: local_state.is_debit.read(),
        is_permanent_account: local_state.is_permanent_account.read(),
        account_name: local_state.account_name.read(),
        notes: local_state.notes.read(),
        unit_of_measurement_of_quantity: local_state.unit_of_measurement_of_quantity.read(),
        belong_to_company: model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .selected_company
            .read()
            .unwrap(),
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
                                process_name: process_manager::ProcessName::CreateAccount,
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
                                process_name: process_manager::ProcessName::CreateAccount,
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
            process_name: process_manager::ProcessName::CreateAccount,
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
            handle_clean::<Rn, Rt, Id, Mpsc, Rg, As>(model);
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
        .page_home
        .page_create_account;

    let user_uuid = commander_local_state.user_uuid.read().clone().unwrap();

    let input = cases::create_account::Input {
        user_uuid,
        new_uuid: Id::generate(),
        is_debit: local_state.is_debit.read(),
        is_permanent_account: local_state.is_permanent_account.read(),
        account_name: local_state.account_name.read(),
        notes: local_state.notes.read(),
        unit_of_measurement_of_quantity: local_state.unit_of_measurement_of_quantity.read(),
        belong_to_company: model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .selected_company
            .read()
            .unwrap(),
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
                    local_state
                        .account_name_error
                        .set(match business_error.account_name {
                            Some(_) => todo!(),
                            None => None,
                        });
                }
            }
        }
    }
}
