use crate::prelude::*;

pub fn new<
    As: AllSignalTypes + 'static,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
>() -> (ui_model::Model<As>, ui_effect::Commander<As, At, Mpsc>) {
    let (sender_to_network, receiver_to_network) = Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = Mpsc::channel();
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::Network::<At, Mpsc>::network_actor(
        receiver_to_network,
        sender_to_cache.clone(),
        sender_to_error.clone(),
        is_online.clone(),
        format!("ws://{}/ws", ADDRESS),
    );

    let cache = cache_actor::Cache::<At, Mpsc>::new(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let model = ui_model::Model::default();

    let sender_to_process_manager = process_manager::process_manager_actor::<As, At, Mpsc>();

    let commander = ui_effect::Commander::new(
        receiver_to_error,
        sender_to_process_manager,
        model.clone(),
        cache,
    );

    (model, commander)
}
