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
        cases::{self, utility::types},
        request_response,
    },
    utility::{traits, utils::ReadAndSet},
};
use std::{str::FromStr, sync::Arc};

pub(crate) type Type1 = cases::create_company::Input;
type Type2 = cases::create_company::Input;
type Type3 = cases::create_company::MyResult;
pub(crate) type Type4 = cases::create_company::MyResult;

impl Into<Vec<types::ResourceInfo>> for &cases::create_company::Ok {
    fn into(self) -> Vec<types::ResourceInfo> {
        let company_uuid = self.new_uuid.clone();

        vec![
            // Company fields
            types::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: types::Resource::TableCompanyFieldName(self.company_name.clone()),
            },
            types::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: types::Resource::TableCompanyFieldCurrency(self.currency.clone()),
            },
            // Access control fields (using the same UUID as the row identifier)
            types::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: types::Resource::TableAccessControlForCompanyFieldRole(self.role.clone()),
            },
            types::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: types::Resource::TableAccessControlForCompanyFieldUser(
                    self.user_uuid.clone(),
                ),
            },
            types::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: types::Resource::TableAccessControlForCompanyFieldDataGroup(company_uuid),
            },
        ]
    }
}

impl ViewType1 for Type1 {
    fn wrap_input(self) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateCompany(self)
    }
}

impl CacheAndServerType1 for Type2 {
    fn user_uuid(&self) -> Option<&types::UuidType> {
        Some(&self.user_uuid)
    }

    type Output = Type3;

    async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
        &self,
        _: &mut cache::State<Ch>,
    ) -> Self::Output {
        let result = self.state_less_operation();
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
        request_response::push_data::OperationsResult::CreateCompany(self)
    }
}

impl ViewType2 for Type4 {
    fn unwrap_output(result: request_response::push_data::OperationsResult) -> Self {
        if let request_response::push_data::OperationsResult::CreateCompany(result) = result {
            return result;
        }
        unreachable!("{:?}", result)
    }
}

impl Mvu for ui_model::CreateCompany {
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
        let page_create_company = &model
            .page_root
            .page_after_auth
            .page_company_branch_selection
            .page_create_company;

        match self {
            Self::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As>(model, cache, commander_local_state).await
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
    Id: types::RowId,
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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
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
