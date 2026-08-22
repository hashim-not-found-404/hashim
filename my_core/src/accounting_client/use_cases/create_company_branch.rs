use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::mbg;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::create_company_branch::Input;
type Type2 = cases::create_company_branch::Input;
type Type3 = cases::create_company_branch::MyResult;
type Type4 = cases::create_company_branch::MyResult;
type StorableType = cases::create_company_branch::Ok;

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
{
    type StorableType = StorableType;
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateCompanyBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut Ch,
    ) -> Self::Type3 {
        let errr = data.state_full_check::<LongCache>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        Ok(data.state_less_operation())
    }

    fn store_resource(data: &Self::Type3) -> Option<Self::StorableType> {
        match data {
            Ok(ok) => Some(ok.clone()),
            Err(_) => None,
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateCompanyBranch(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        let local_state = &model.page_create_company_branch;

        match output {
            Ok(_) => {
                local_state.branch_name_error.reset();
                local_state.location_error.reset();
            }
            Err(business_error) => {
                mbg!(business_error);
                todo!();
                // local_state.branch_name_error.set(todo!());
                // local_state.location_error.set(todo!());
            }
        }
    }
}

impl ui_model::CreateCompanyBranch {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await
            }
            Self::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::CreateCompanyBranch,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            Self::Close => handle_close::<As>(model),
            Self::Name(i) => {
                model.page_create_company_branch.branch_name.set(i);

                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            Self::Currency(i) => model.page_create_company_branch.currency.set(i),
        }
    }
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let local_state = &model.page_create_company_branch;

    if local_state.is_loading.read() {
        return;
    }
    local_state.is_loading.set(true);

    let input = build_input::<Id, As>(model);
    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_company_branch.show_dialog,
        process_manager::ProcessName::CreateCompanyBranch,
        data,
        move |data| {
            let result = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);

            let is_ok = result.is_ok();
            if is_ok {
                handle_close::<As>(model);
            }

            is_ok
        },
    )
    .await;

    local_state.is_loading.reset();
}

async fn handle_check<
    Rn: traits::RandomNumber,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let input = build_input::<Id, As>(model);

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            Rn::generate(),
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input),
        )
        .await;

    match receiver_to_response.recv().await.unwrap() {
        cache_actor::Response::CloseTheChannel => {}
        cache_actor::Response::ServerCannotBeReached => {}
        cache_actor::Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result: Type4 =
                <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);

            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);
        }
    }
}

fn handle_close<As: ui_model::AllSignalTypes>(model: &'static ui_model::Model<As>) {
    let page_create_company_branch = &model.page_create_company_branch;

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
        .set(ui_model::Navigator::ListCompanyAndBranch(ui_model::ListCompanyAndBranch::None));
}

fn build_input<Id: types::RowId, As: ui_model::AllSignalTypes>(
    model: &ui_model::Model<As>,
) -> Type1 {
    let local_state = &model.page_create_company_branch;

    cases::create_company_branch::Input {
        user_uuid:      model.user_uuid.read().clone().unwrap(),
        new_uuid:       Id::generate(),
        company_belong: model.selected_company.read().unwrap(),
        currency:       local_state.currency.read(),
        branch_name:    local_state.branch_name.read(),
        location:       local_state.location.read(),
    }
}
