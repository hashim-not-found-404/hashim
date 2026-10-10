use crate::model::TypeModel;
use crate::wire::MyCaster;
use cache::cache_adapter;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MultiProducerSingleConsumer;
use kernel::ui_construct::new;
use std::sync::Arc;
use std::sync::LazyLock;
use use_case_error_handler::client::spawn_listener;
use utility::cache::ProcessId;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;

pub(crate) static MODEL: LazyLock<Arc<TypeModel>> =
    LazyLock::new(|| -> Arc<TypeModel> { Arc::new(TypeModel::default()) });

static COMMANDER: LazyLock<Commander> = LazyLock::new(|| {
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let model = MODEL.to_owned();

    let commander =
        new::<cache_adapter::S, TypeModel, MyCaster, MyCaster, MyCaster>(model, sender_to_error);

    spawn_listener(receiver_to_error, commander.clone());

    commander
});

pub(crate) fn send<Msg: MessageTrait>(process_id: ProcessId, msg: Msg) {
    COMMANDER.send(process_id, msg);
}

pub(crate) fn init_commander_and_model() {
    LazyLock::force(&MODEL);
    LazyLock::force(&COMMANDER);
}
