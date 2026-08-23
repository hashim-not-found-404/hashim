use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor;
use crate::client::utility::client_traits;
use crate::client::utility::commander;
use crate::client::utility::process_manager;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::utility::types::JsonWebTokenType;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::MakeOptionIfEmpty;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = use_cases::sign_up::Input;
type Type2 = use_cases::sign_up::Input;
type Type3 = use_cases::sign_up::MyResult;
type Type4 = use_cases::sign_up::MyResult;

make_wrap_unwrap!(sign_up, SignUp);
make_user_uuid!(sign_up);

pub(crate) async fn state_full_operation<
    Ch: Cache,
    LongCache: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Ch>,
>(
    data: &Type2,
    state: &mut Ch,
) -> Type3 {
    let errr = data.state_full_check::<LongCache>(state).await.unwrap();

    if errr.is_there_error() {
        return Err(errr);
    }

    Ok(use_cases::sign_up::Ok {
        new_uuid:        data.user_uuid.clone(),
        user_id:         data.user_id.clone(),
        user_name:       data.name.clone(),
        hashed_password: String::new(),
        jwt:             JsonWebTokenType(String::new()),
    })
}

pub(crate) fn extract_resource(data: &Type3) -> Vec<resource_utils::ResourceInfo> {
    match data {
        Ok(ok) => {
            let mut resource = Vec::with_capacity(3);

            resource.push(resource_utils::ResourceInfo {
                row_uuid: ok.new_uuid.0.clone(),
                resource: resource_utils::Resource::Jwt(ok.jwt.clone()),
            });

            resource.push(resource_utils::ResourceInfo {
                row_uuid: ok.new_uuid.0.clone(),
                resource: resource_utils::Resource::TableUserFieldId(ok.user_id.clone()),
            });

            if let Some(user_name) = &ok.user_name {
                resource.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.0.clone(),
                    resource: resource_utils::Resource::TableUserFieldName(user_name.clone()),
                });
            }

            resource
        }
        Err(_) => Vec::new(),
    }
}

fn apply_on_the_model<As: ui_model::AllSignalTypes>(output: &Type4, model: &ui_model::Model<As>) {
    let local_state = &model.page_sign_up;
    match output {
        Ok(_) => {
            local_state.user_id_error.reset();
            local_state.user_name_error.reset();
        }
        Err(business_error) => {
            local_state
                .user_id_error
                .set(business_error.user_id.as_ref().map(|_| String::from("duplicated user")));
            local_state.user_name_error.set(business_error.name.clone());
        }
    }
}

impl ui_model::SignUp {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: Cache,
        LongCache: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::GoToSignIn => {
                if !model.feature_state_auth.is_loading.read() {
                    model.navigator.set(ui_model::Navigator::SignIn);
                }
            }
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
                        process_name: process_manager::ProcessName::SignUp,
                        consent:      i,
                    })
                    .await
                    .unwrap()
            }
            Self::UserName(i) => {
                model.user_name.set(i);
                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            Self::UserId(i) => {
                model.user_id.set(i);
                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            Self::Password(i) => {
                model.feature_state_auth.user_password.set(i);
                handle_check::<Rn, Id, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
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
    LongCache: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.feature_state_auth;
    let local_state = &model.page_sign_up;

    if feature_state.is_loading.read() {
        return;
    }
    feature_state.is_loading.set(true);

    local_state.show_dialog.reset();
    local_state.user_id_error.reset();
    local_state.user_name_error.reset();

    let new_uuid: User = Id::generate().into();
    let input = build_input::<As>(model, new_uuid.clone());

    let data = wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_sign_up.show_dialog,
        process_manager::ProcessName::SignUp,
        data,
        move |data| {
            let result = unwrap_output(data);
            apply_on_the_model(&result, model);

            let is_ok = result.is_ok();
            if is_ok {
                model.user_uuid.put(Some(new_uuid.clone()));

                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::None,
                ));
            }

            is_ok
        },
    )
    .await;

    feature_state.is_loading.reset();
}

async fn handle_check<
    Rn: traits::RandomNumber,
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let local_state = &model.page_sign_up;

    local_state.user_id_error.reset();
    local_state.user_name_error.reset();

    let input = build_input::<As>(model, Id::generate().into());
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

fn build_input<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>, new_uuid: User) -> Type1 {
    use_cases::sign_up::Input {
        user_uuid: new_uuid,
        name:      model.user_name.read().none_if_empty(),
        user_id:   model.user_id.read(),
        password:  model.feature_state_auth.user_password.read(),
    }
}
