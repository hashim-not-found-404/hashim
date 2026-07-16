use crate::{
    accounting_client::{
        network_actor, ui_effect,
        use_cases::client_domain::{cache, cache_actor, client_traits, process_manager, ui_model},
    },
    accounting_domain::{cases, types},
    utility::traits,
};
use std::sync::{Arc, RwLock};

pub fn new<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: cases::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ed: traits::Coding,
    Rg: traits::Regex,
    Ch: cache::Cache,
    Ws: network_actor::WSClient,
    As: ui_model::AllSignalTypes,
>(
    model: &'static ui_model::Model<As>,
) -> ui_effect::Commander<Mpsc> {
    let (sender_to_network, receiver_to_network) = Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = Mpsc::channel();
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::network_actor::<Rt, Ws>(format!("ws://{}/ws", types::ADDRESS));

    let cache = client_traits::Type::new::<Rt, Ed, Rn>(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager::process_manager_actor::<Mpsc, As, Rt>();

    let commander = ui_effect::Commander::new::<As, Rt, Rn, Id, Rg>(
        receiver_to_error,
        sender_to_process_manager,
        model,
        cache,
    );

    commander
}
