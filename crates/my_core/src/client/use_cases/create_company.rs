use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor;
use crate::client::utility::client_traits;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::utility::types::RowId;
use crate::make_wrap_unwrap;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;

type Type1 = use_cases::create_company::Input;
type Type2 = use_cases::create_company::Input;
type Type3 = use_cases::create_company::MyResult;
type Type4 = use_cases::create_company::MyResult;

make_wrap_unwrap!(create_company, CreateCompany);

pub(crate) async fn state_full_operation(data: &Type2) -> Type3 {
    Ok(data.state_less_operation())
}

impl ui_model::CreateCompany {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: Cache,
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
    Ch: Cache,
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
            wrap_input(input.clone()),
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
