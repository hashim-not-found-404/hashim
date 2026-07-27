use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::create_company::Input;
type Type2 = cases::create_company::Input;
type Type3 = cases::create_company::MyResult;
type Type4 = cases::create_company::MyResult;

pub(crate) struct ViewAndCacheType;

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
        _: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let result = data.state_less_operation();
        return Ok(result);
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let this = ok;
                let company_uuid = this.new_uuid.clone();
                vec![
                    // Company fields
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldName(
                            this.company_name.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldCurrency(
                            this.currency.clone(),
                        ),
                    },
                    // Access control fields (using the same UUID as the row identifier)
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableAccessControlForCompanyFieldRole(
                            this.role.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource: resource_utils::Resource::TableAccessControlForCompanyFieldUser(
                            this.user_uuid.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.clone(),
                        resource:
                            resource_utils::Resource::TableAccessControlForCompanyFieldDataGroup(
                                company_uuid,
                            ),
                    },
                ]
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateCompany(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(_: &Self::Type4, _: &ui_model::Model<As>) {
        todo!()
    }
}

impl ui_model::CreateCompany {
    pub(crate) async fn update<
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
        let local_state = &model.page_create_company;

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
            Self::Name(i) => local_state.company_name.set(i),
            Self::Currency(i) => local_state.currency.set(i),
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
    let page_create_company = &model.page_create_company;

    page_create_company.company_name.reset();
    page_create_company.currency.reset();

    model
        .navigator
        .set(ui_model::Navigator::ListCompanyAndBranch(ui_model::ListCompanyAndBranch::None));
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
    _: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let data = model.user_uuid.read().clone().unwrap();

    let local_state = &model.page_create_company;

    let input = cases::create_company::Input {
        user_uuid:    data,
        new_uuid:     Id::generate(),
        company_name: local_state.company_name.read(),
        currency:     local_state.currency.read(),
    };

    let txn_number = Rn::generate();

    cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::WriteCacheAndServer,
            txn_number,
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input.clone()),
        )
        .await;

    handle_close::<Rn, Rt, Id, Mpsc, Rg, As>(model);
}
