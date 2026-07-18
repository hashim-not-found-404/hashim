use crate::{
    accounting_client::use_cases::client_domain::{
        cache, cache_actor,
        client_traits::{
            self, CacheAndServerType1, CacheAndServerType2, Mvu, ViewType1, ViewType2,
        },
        commander,
        ui_model::{self, HashimSignal},
    },
    accounting_domain::{
        cases::{
            self,
            utility::types::{self, MyErrorTrait},
        },
        request_response,
    },
    utility::{traits, utils::ReadAndSet},
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
        let page_create_account = &model
            .page_root
            .page_after_auth
            .page_home
            .page_create_account;

        match self {
            ui_model::CreateAccount::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await
            }
            ui_model::CreateAccount::Close => todo!(),
            ui_model::CreateAccount::IsDebit(v) => page_create_account.is_debit.set(v),
            ui_model::CreateAccount::IsPermanentAccount(v) => {
                page_create_account.is_permanent_account.set(v)
            }
            ui_model::CreateAccount::AccountName(v) => page_create_account.account_name.set(v),
            ui_model::CreateAccount::Notes(v) => page_create_account.notes.set(v),
            ui_model::CreateAccount::UnitOfMeasurementOfQuantity(v) => {
                page_create_account.unit_of_measurement_of_quantity.set(v)
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
    todo!();
    let page_create_account = &model
        .page_root
        .page_after_auth
        .page_company_branch_selection;
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
    todo!();
    let data = commander_local_state.user_uuid.read().clone().unwrap();

    let local_state = &model
        .page_root
        .page_after_auth
        .page_company_branch_selection;

    let input = cases::create_account::Input {
        user_uuid: todo!(),
        new_uuid: todo!(),
        is_debit: todo!(),
        is_permanent_account: todo!(),
        account_name: todo!(),
        notes: todo!(),
        unit_of_measurement_of_quantity: todo!(),
        belong_to_company: todo!(),
    };

    cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::WriteCacheAndServer,
            input.clone().wrap_input(),
        )
        .await;

    handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model);
}
