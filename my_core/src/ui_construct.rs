use crate::prelude::*;

pub fn new<At: AllClientTypes + 'static>() -> (ui_model::Model<At>, ui_effect::Commander<At>) {
    let (sender_to_network, receiver_to_network) = At::Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = At::Mpsc::channel();
    let (sender_to_error, receiver_to_error) = At::Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::Network::<At>::network_actor(
        receiver_to_network,
        sender_to_cache.clone(),
        sender_to_error.clone(),
        is_online.clone(),
        format!("ws://{}/ws", ADDRESS),
    );

    let cache = cache_actor::Cache::<At>::new(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let model = ui_model::Model::default();

    let sender_to_process_manager = process_manager::process_manager_actor::<At>();

    let commander = ui_effect::Commander::new(
        receiver_to_error,
        sender_to_process_manager,
        model.clone(),
        cache,
    );

    (model, commander)
}
