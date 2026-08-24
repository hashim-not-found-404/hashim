use crate::client::utility::cache_actor::CachingStrategy;
use crate::client::utility::client_traits::CacheActorStruct;
use crate::client::utility::client_traits::Subscribe;
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

const SUBS_TO_POKE: &'static [Subscribe] = &[];

make_wrap_unwrap!(get_all_accounts, GetAllAccounts);
make_user_uuid!(get_all_accounts);

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
