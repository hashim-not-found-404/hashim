use crate::model::TypeModel;
use crate::wire::MyCaster;
use cache::cache_adapter;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MultiProducerSingleConsumer;
use kernel::ui_construct;
use std::sync::Arc;
use std::sync::LazyLock;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;

pub(crate) static MODEL: LazyLock<Arc<TypeModel>> =
    LazyLock::new(|| -> Arc<TypeModel> { Arc::new(TypeModel::default()) });

pub(crate) static COMMANDER: LazyLock<Commander> = LazyLock::new(|| {
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let model = MODEL.to_owned();

    use_case_error_handler::client::spawn_listener(
        receiver_to_error,
        model.page_error_handler.clone(),
    );

    ui_construct::new::<cache_adapter::S, TypeModel, MyCaster, MyCaster, MyCaster>(
        model,
        sender_to_error,
    )
});

pub(crate) fn send<Msg: MessageTrait>(msg: Msg) {
    COMMANDER.send(msg);
}
