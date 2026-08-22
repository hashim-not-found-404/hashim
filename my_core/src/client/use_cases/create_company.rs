use crate::client::utility::cache;
use crate::client::utility::cache_actor;
use crate::client::utility::client_traits;
use crate::client::utility::client_traits::ViewAndCache;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::request_response;
use crate::domain::use_cases;
use crate::domain::utility::resource_utils;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::User;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;

type Type1 = use_cases::create_company::Input;
type Type2 = use_cases::create_company::Input;
type Type3 = use_cases::create_company::MyResult;
type Type4 = use_cases::create_company::MyResult;

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> use_cases::create_company::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::OperationsInput {
        request_response::OperationsInput::CreateCompany(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&User> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, _: &mut Ch) -> Self::Type3 {
        Ok(data.state_less_operation())
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let this = ok;
                let company_uuid = this.new_uuid.clone();
                vec![
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldName(
                            this.company_name.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableCompanyFieldCurrency(
                            this.currency.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccessControlForCompanyFieldRole(
                            this.role.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccessControlForCompanyFieldUser(
                            this.user_uuid.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: company_uuid.0.clone(),
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

    fn unwrap_output(output: request_response::OperationsResult) -> Self::Type4 {
        if let request_response::OperationsResult::CreateCompany(result) = output {
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
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> use_cases::create_company::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
    ) {
        let local_state = &model.page_create_company;

        match self {
            Self::Submit => handle_submit::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await,
            Self::Close => handle_close::<As>(model),
            Self::Name(i) => local_state.company_name.set(i),
            Self::Currency(i) => local_state.currency.set(i),
        }
    }
}

fn handle_close<As: ui_model::AllSignalTypes>(model: &'static ui_model::Model<As>) {
    let page_create_company = &model.page_create_company;

    page_create_company.company_name.reset();
    page_create_company.currency.reset();

    model
        .navigator
        .set(ui_model::Navigator::ListCompanyAndBranch(ui_model::ListCompanyAndBranch::None));
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> use_cases::create_company::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let input = build_input::<Id, As>(model);
    cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::WriteCacheAndServer,
            Rn::generate(),
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input.clone()),
        )
        .await;

    handle_close::<As>(model);
}

fn build_input<Id: RowId, As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) -> Type1 {
    let local_state = &model.page_create_company;

    use_cases::create_company::Input {
        user_uuid:    model.user_uuid.read().clone().unwrap(),
        new_uuid:     Id::generate().into(),
        company_name: local_state.company_name.read(),
        currency:     local_state.currency.read(),
    }
}
