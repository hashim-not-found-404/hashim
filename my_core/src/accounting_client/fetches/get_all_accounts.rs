use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ReadServerOnly;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;

type Type1 = cases::get_all_accounts::Input;
type Type2 = cases::get_all_accounts::Input;
type Type3 = cases::get_all_accounts::MyResult;

pub(crate) struct ViewAndCacheType;

impl ReadServerOnly for ViewAndCacheType {
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;

    fn wrap_input(data: Self::Type1) -> request_response::OperationsInput {
        request_response::OperationsInput::GetAllAccounts(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&UuidType> {
        Some(&data.user_uuid)
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();
                for account in &ok.data {
                    let row_uuid = &account.row_uuid;

                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldName(
                            account.account_name.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldCompanyBelong(
                            ok.company_uuid.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldIsDebit(
                            account.is_debit,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldIsPermanentAccount(
                            account.is_permanent_account,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldNotes(
                            account.notes.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: row_uuid.clone(),
                        resource:
                            resource_utils::Resource::TableAccountFieldUnitOfMeasurementOfQuantity(
                                account.unit_of_measurement_of_quantity.clone(),
                            ),
                    });
                }

                resources
            }
            Err(_) => Vec::new(),
        }
    }
}

pub(crate) async fn fetch<
    Rn: traits::RandomNumber,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
>(
    model: &ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
) {
    let company_uuid = model.selected_company.read().unwrap();

    let input = cases::get_all_accounts::Input {
        user_uuid: model.user_uuid.read().clone().unwrap(),
        company_uuid,
    };

    let txn_number = Rn::generate();

    cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadServerOnly,
            txn_number,
            ViewAndCacheType::wrap_input(input),
        )
        .await;
}
