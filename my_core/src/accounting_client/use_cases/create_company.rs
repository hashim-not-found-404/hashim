use crate::{
    accounting_client::use_cases::client_domain::{
        cache, cache_actor,
        client_traits::{self, ViewAndCache},
        commander,
        ui_model::{self, HashimSignal},
    },
    accounting_domain::{
        cases::{
            self,
            utility::{resource_utils, types},
        },
        request_response,
    },
    utility::{traits, utils::ReadAndSet},
};
use std::{marker::PhantomData, str::FromStr, sync::Arc};

pub(crate) type Type1 = cases::create_company::Input;
type Type2 = cases::create_company::Input;
type Type3 = cases::create_company::MyResult;
pub(crate) type Type4 = cases::create_company::MyResult;

impl Into<Vec<resource_utils::ResourceInfo>> for &cases::create_company::Ok {
    fn into(self) -> Vec<resource_utils::ResourceInfo> {
        let company_uuid = self.new_uuid.clone();

        vec![
            // Company fields
            resource_utils::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: resource_utils::Resource::TableCompanyFieldName(
                    self.company_name.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: resource_utils::Resource::TableCompanyFieldCurrency(
                    self.currency.clone(),
                ),
            },
            // Access control fields (using the same UUID as the row identifier)
            resource_utils::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: resource_utils::Resource::TableAccessControlForCompanyFieldRole(
                    self.role.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: resource_utils::Resource::TableAccessControlForCompanyFieldUser(
                    self.user_uuid.clone(),
                ),
            },
            resource_utils::ResourceInfo {
                row_uuid: company_uuid.clone(),
                resource: resource_utils::Resource::TableAccessControlForCompanyFieldDataGroup(
                    company_uuid,
                ),
            },
        ]
    }
}

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::create_company::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_company::ReadInput,
    ) -> Result<cases::create_company::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await.unwrap();
        Ok(read_output)
    }
}

struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateCompany(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let result = data.state_less_operation();
        return Ok(result);
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => ok.into(),
            Err(_) => Vec::new(),
        }
    }

    fn wrap_output(data: Self::Type3) -> request_response::push_data::OperationsResult {
        request_response::push_data::OperationsResult::CreateCompany(data)
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateCompany(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }
}

impl ui_model::CreateCompany {
    async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>,
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
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
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
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>,
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
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input.clone()),
        )
        .await;

    handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model);
}
