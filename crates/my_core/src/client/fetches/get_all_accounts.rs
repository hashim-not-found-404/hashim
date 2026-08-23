use crate::client::utility::cache_actor::CachingStrategy;
use crate::client::utility::client_traits::CacheActorStruct;
use crate::client::utility::ui_model::AllSignalTypes;
use crate::client::utility::ui_model::Model;
use crate::domain::use_cases::get_all_accounts::Input;
use crate::domain::use_cases::get_all_accounts::MyResult;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::RandomNumber;
use crate::utility::utils::ReadAndSet;

type Type3 = MyResult;

make_wrap_unwrap!(get_all_accounts, GetAllAccounts);
make_user_uuid!(get_all_accounts);

pub(crate) fn extract_resource(data: &Type3) -> Vec<resource_utils::ResourceInfo> {
    match data {
        Ok(ok) => {
            let mut resources = Vec::new();
            for account in &ok.data {
                let row_uuid = &account.row_uuid;

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: row_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldName(
                        account.account_name.clone(),
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: row_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldCompanyBelong(
                        ok.company_uuid.clone(),
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: row_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldIsDebit(account.is_debit),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: row_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldIsPermanentAccount(
                        account.is_permanent_account,
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: row_uuid.0.clone(),
                    resource: resource_utils::Resource::TableAccountFieldNotes(
                        account.notes.clone(),
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: row_uuid.0.clone(),
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

pub(crate) async fn fetch<
    Rn: RandomNumber,
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
>(
    model: &Model<As>,
    mut cache: CacheActorStruct<Mpsc>,
) {
    let company_uuid = model.selected_company.read().unwrap();

    let input = Input {
        user_uuid: model.user_uuid.read().clone().unwrap(),
        company_uuid,
    };

    let txn_number = Rn::generate();

    cache.send_to_cache_actor(CachingStrategy::ReadServerOnly, txn_number, wrap_input(input)).await;
}
