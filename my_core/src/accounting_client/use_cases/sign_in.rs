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
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::sign_in::Input;
type Type2 = cases::sign_in::Input;
type Type3 = cases::sign_in::MyResult;
type Type4 = Result<SignInOk, cases::sign_in::Error>;

pub(crate) struct SignInOk {
    user_uuid: types::UuidType,
    user_name: String,
}

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::SignIn(data)
    }

    fn user_uuid(_: &Self::Type2) -> Option<&types::UuidType> {
        None
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let read_output = LongCache::read(&mut state.cache, &cases::sign_in::ReadInput {
            user_id: data.user_id.clone(),
        })
        .await
        .unwrap();

        if let Some((user_uuid, jwt, user_name)) = read_output.user_rowid_and_password_hash_and_name
            && !jwt.is_empty()
        {
            return Ok(cases::sign_in::Ok {
                user_uuid,
                jwt: types::JsonWebTokenType(String::new()),
                user_id: data.user_id.clone(),
                user_name,
            });
        }

        let mut password = None;
        let mut user_uuid = None;
        let mut user_name = None;

        for (rowid, user) in &state.state_of_pending_txn.user {
            if user.id == data.user_id {
                password = Some(user.password.clone());
                user_uuid = Some(rowid);
                user_name = user.name.clone();
            }
        }

        match password {
            Some(password) => {
                if password == data.password {
                    Ok(data.state_full_operation(
                        &types::JsonWebTokenType(String::new()),
                        user_uuid.unwrap(),
                        user_name.as_ref(),
                    ))
                } else {
                    Err(cases::sign_in::Error {
                        user_id:  None,
                        password: Some(cases::sign_in::PasswordError::WrongPassword),
                    })
                }
            }
            None => {
                Err(cases::sign_in::Error {
                    user_id:  Some(cases::sign_in::UserIdError::NotExist),
                    password: None,
                })
            }
        }
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::with_capacity(3);
                let user_uuid = &ok.user_uuid;

                // JWT
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: user_uuid.clone(),
                    resource: resource_utils::Resource::Jwt(ok.jwt.clone()),
                });

                // User ID
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: user_uuid.clone(),
                    resource: resource_utils::Resource::TableUserFieldId(ok.user_id.clone()),
                });

                // User name (optional)
                if let Some(name) = &ok.user_name {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: user_uuid.clone(),
                        resource: resource_utils::Resource::TableUserFieldName(name.clone()),
                    });
                }

                resources
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::SignIn(result) = output {
            match result {
                Ok(ok) => {
                    Ok(SignInOk {
                        user_uuid: ok.user_uuid,
                        user_name: ok.user_name.unwrap_or_default(),
                    })
                }
                Err(err) => Err(err),
            }
        } else {
            unreachable!("{:?}", output)
        }
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        match output {
            Ok(ok) => {
                model.user_name.set(ok.user_name.clone());
            }
            Err(business_error) => {
                model
                    .page_sign_in
                    .user_id_error
                    .set(business_error.user_id.as_ref().map(|_| String::from("user not exist")));

                model
                    .page_sign_in
                    .user_password_error
                    .set(business_error.password.as_ref().map(|_| String::from("wrong password")));
            }
        }
    }
}

impl ui_model::SignIn {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache,
        LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::GoToSignUp => {
                if !model.feature_state_auth.is_loading.read() {
                    model.navigator.set(ui_model::Navigator::SignUp);
                }
            }
            Self::Submit => {
                handle_submit::<Rn, Rt, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            Self::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::SignIn,
                        consent:      i,
                    })
                    .await
                    .unwrap()
            }
            Self::UserId(i) => {
                model.user_id.set(i);
                handle_check::<Rn, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
            Self::Password(i) => {
                model.feature_state_auth.user_password.set(i);
                handle_check::<Rn, Mpsc, As, Ch, LongCache>(model, cache).await;
            }
        }
    }
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let feature_state = &model.feature_state_auth;
    let local_state = &model.page_sign_in;

    if feature_state.is_loading.read() {
        return;
    }
    feature_state.is_loading.set(true);

    local_state.show_dialog.reset();
    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let input = build_input::<As>(model);
    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_sign_in.show_dialog,
        process_manager::ProcessName::SignIn,
        data,
        move |data| {
            let result = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);

            let is_ok = result.is_ok();
            if let Ok(ok) = result {
                model.user_uuid.put(Some(ok.user_uuid));

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
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let local_state = &model.page_sign_in;

    local_state.user_id_error.reset();
    local_state.user_password_error.reset();

    let input = build_input::<As>(model);

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
            handle_apply_result::<As, Ch, LongCache>(model, result);
        }
    }
}

fn handle_apply_result<
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>,
>(
    model: &ui_model::Model<As>,
    result: Type4,
) {
    <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);
    if let Ok(ok) = result {
        model.user_uuid.put(Some(ok.user_uuid));
    }
}

fn build_input<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) -> Type1 {
    cases::sign_in::Input {
        user_id:  model.user_id.read(),
        password: model.feature_state_auth.user_password.read(),
    }
}
