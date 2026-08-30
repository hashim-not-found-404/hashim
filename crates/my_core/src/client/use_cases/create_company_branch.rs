use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor;
use crate::client::utility::client_traits;
use crate::client::utility::commander;
use crate::client::utility::process_manager;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::make_wrap_unwrap;
use crate::mbg;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = use_cases::create_company_branch::Input;
type Type2 = use_cases::create_company_branch::Input;
type Type3 = use_cases::create_company_branch::MyResult;
type Type4 = use_cases::create_company_branch::MyResult;

make_wrap_unwrap!(create_company_branch, CreateCompanyBranch);

pub(crate) async fn state_full_operation<
    Ch: Cache,
    LongCache: for<'a> use_cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
>(
    data: &Type2,
    state: &mut Ch,
) -> Type3 {
    let errr = data.state_full_check::<LongCache>(state).await.unwrap();

    if errr.is_there_error() {
        return Err(errr);
    }

    Ok(data.state_less_operation())
}

fn apply_on_the_model<As: ui_model::AllSignalTypes>(output: &Type4, model: &ui_model::Model<As>) {
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

impl ui_model::CreateCompanyBranch {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: Cache,
        LongCache: for<'a> use_cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
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
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
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
    let data = wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_company_branch.show_dialog,
        process_manager::ProcessName::CreateCompanyBranch,
        data,
        move |data| {
            let result = unwrap_output(data);
            apply_on_the_model(&result, model);

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
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::create_company_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let input = build_input::<Id, As>(model);

    let mut receiver_to_response = cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadCacheOnly,
            Rn::generate(),
            wrap_input(input),
        )
        .await;

    match receiver_to_response.recv().await.unwrap() {
        cache_actor::Response::CloseTheChannel => {}
        cache_actor::Response::ServerCannotBeReached => {}
        cache_actor::Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result: Type4 = unwrap_output(data);

            apply_on_the_model(&result, model);
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
        .set(ui_model::Navigator::GetCompaniesAndBranches(ui_model::GetCompaniesAndBranches::None));
}

fn build_input<Id: RowId, As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) -> Type1 {
    let local_state = &model.page_create_company_branch;

    use_cases::create_company_branch::Input {
        user_uuid:      model.user_uuid.read().clone().unwrap(),
        new_uuid:       Id::generate().into(),
        company_belong: model.selected_company.read().unwrap(),
        currency:       local_state.currency.read(),
        branch_name:    local_state.branch_name.read(),
        location:       local_state.location.read(),
    }
}
