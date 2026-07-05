use crate::{
    accounting_client::{
        cache_actor, client_traits::AllClientTypes, network_actor, process_manager, ui_effect,
        ui_model,
    },
    accounting_domain::db_types,
    utility::traits::MultiProducerSingleConsumer,
};
use std::sync::{Arc, RwLock};

struct Abc {}
impl network_actor::Network for Abc {
    async fn from_network_status(&mut self, are_we_online: bool) {
        todo!()
    }

    async fn sender_to_network(&mut self, data: Vec<u8>) {
        todo!()
    }
}

pub fn new<At: AllClientTypes>(model: &'static ui_model::Model<At>) -> ui_effect::Commander<At> {
    let (sender_to_network, receiver_to_network) = At::Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = At::Mpsc::channel();
    let (sender_to_error, receiver_to_error) = At::Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::network_actor::<At, Abc>(
        receiver_to_network,
        Abc {},
        sender_to_error.clone(),
        is_online.clone(),
        format!("ws://{}/ws", db_types::ADDRESS),
    );

    let cache = cache_actor::CacheStruct::<At>::new(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager::process_manager_actor::<At>();

    let commander =
        ui_effect::Commander::new(receiver_to_error, sender_to_process_manager, model, cache);

    commander
}
