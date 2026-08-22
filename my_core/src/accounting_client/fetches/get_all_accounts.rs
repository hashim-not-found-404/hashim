use crate::accounting_client::client_domain::cache_actor::CachingStrategy;
use crate::accounting_client::client_domain::client_traits::CacheActorStruct;
use crate::accounting_client::client_domain::client_traits::ReadServerOnly;
use crate::accounting_client::client_domain::ui_model::AllSignalTypes;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_client::client_domain::ui_model::Model;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response::push_data::OperationsInput;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::RandomNumber;
use crate::utility::utils::ReadAndSet;

type Type1 = cases::get_all_accounts::Input;
type Type2 = cases::get_all_accounts::Input;
type Type3 = cases::get_all_accounts::MyResult;
type StorableType = cases::get_all_accounts::Ok;

pub(crate) struct ViewAndCacheType;

impl ReadServerOnly for ViewAndCacheType {
    type StorableType = StorableType;
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;

    fn wrap_input(data: Self::Type1) -> OperationsInput {
        OperationsInput::GetAllAccounts(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&UuidType> {
        Some(&data.user_uuid)
    }

    fn extract_resource(data: &Self::Type3) -> Option<Self::StorableType> {
        match data {
            Ok(ok) => Some(ok.clone()),
            Err(_) => None,
        }
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

    let input = cases::get_all_accounts::Input {
        user_uuid: model.user_uuid.read().clone().unwrap(),
        company_uuid,
    };

    let txn_number = Rn::generate();

    cache
        .send_to_cache_actor(
            CachingStrategy::ReadServerOnly,
            txn_number,
            ViewAndCacheType::wrap_input(input),
        )
        .await;
}
