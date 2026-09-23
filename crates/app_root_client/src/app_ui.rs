use crate::model::TypeModel;
use crate::navigator::Navigator;
use crate::wire::MyCaster;
use cache::cache_adapter;
use dioxus::prelude::*;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MultiProducerSingleConsumer;
use kernel::ui_construct;
use std::sync::Arc;
use std::sync::LazyLock;
use use_case_error_handler::client::LocalModel;
use use_case_error_handler::ui::ErrorStack;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility_ui::domain::HashimSignal;

static MODEL: LazyLock<Arc<TypeModel>> =
    LazyLock::new(|| -> Arc<TypeModel> { Arc::new(TypeModel::default()) });

static COMMANDER: LazyLock<Commander> = LazyLock::new(|| {
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

#[derive(Debug, Clone, PartialEq, Routable)]
pub(crate) enum Route {
    #[layout(RootLayout)]
    // #[route("/")]
    // SignIn {},
    // #[route("/sign_up")]
    // SignUp {},
    // #[route("/get_companies_and_branches")]
    // GetCompaniesAndBranches {},
    #[route("/")]
    Home {},
}

#[component]
fn Home() -> Element {
    rsx! {}
}

#[component]
fn RootLayout() -> Element {
    match MODEL.navigator.read() {
        Navigator::SignIn => {
            // navigator().push(Route::SignIn {});
        }
        Navigator::SignUp => {
            // navigator().push(Route::SignUp {});
        }
        Navigator::GetCompaniesAndBranches(_) => {
            // navigator().push(Route::GetCompaniesAndBranches {});
        }
        Navigator::Home(_) => {
            navigator().push(Route::Home {});
        }
    }

    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn App() -> Element {
    LazyLock::force(&MODEL);
    LazyLock::force(&COMMANDER);

    rsx! {
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
        ErrorStack {
            close_error_callback: move |msg| {
                send(msg);
            },
            is_expand_all: MODEL.page_error_handler.is_expand_all().read(),
            errors: MODEL.page_error_handler.errors().read(),
        }
    }
}
