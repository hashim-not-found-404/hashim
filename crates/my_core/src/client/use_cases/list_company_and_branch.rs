use crate::client::utility::cache::Cache;
use crate::client::utility::client_traits;
use crate::client::utility::client_traits::OperationName;
use crate::client::utility::commander;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::utility::types::Branch;
use crate::domain::utility::types::Company;
use crate::domain::utility::types::ListOfCompanies;
use crate::domain::utility::uuid;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = use_cases::list_company_and_branch::Input;
type Type2 = use_cases::list_company_and_branch::Input;
type Type3 = use_cases::list_company_and_branch::MyResult;
type Type4 = use_cases::list_company_and_branch::MyResult;

const LISTEN_TO_OPERATIONS: &'static [OperationName] =
    &[OperationName::CreateCompany, OperationName::CreateCompanyBranch];

make_wrap_unwrap!(list_company_and_branch, ListCompanyAndBranch);
make_user_uuid!(list_company_and_branch);

impl tools::Sortable for Company {
    type Key = (String, uuid::Company);

    fn key(&self) -> Self::Key {
        (self.name.clone(), self.uuid.clone())
    }
}

impl tools::Sortable for Branch {
    type Key = (String, uuid::Branch);

    fn key(&self) -> Self::Key {
        (self.name.clone(), self.uuid.clone())
    }
}

pub fn sort_companies(companies: &mut ListOfCompanies) {
    tools::sort(companies);
    for company in companies {
        tools::sort(&mut company.branches);
    }
}

pub(crate) async fn state_full_operation<
    Ch: Cache,
    LongCache: for<'a> use_cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
>(
    data: &Type2,
    state: &mut Ch,
) -> Type3 {
    let result = data.state_full_operation::<LongCache>(state).await.unwrap();

    Ok(result)
}

fn apply_on_the_model<As: ui_model::AllSignalTypes>(output: &Type4, model: &ui_model::Model<As>) {
    todo!()
}

impl ui_model::CompanyAndBranchSelection {
    pub(crate) fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: Cache + 'static,
        LongCache: for<'a> use_cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            Self::Subscribe => {
                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::None,
                ));

                spawn_listener::<Rn, Rt, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                );
            }
            Self::UnSubscribe => {
                commander_local_state.aborter_to_company_and_branch_listener.abort();
            }
            Self::ShowCreateCompany => {
                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::CreateCompany,
                ));
            }
            Self::ShowCreateCompanyBranch => {
                model.navigator.set(ui_model::Navigator::ListCompanyAndBranch(
                    ui_model::ListCompanyAndBranch::CreateCompanyBranch,
                ));
            }
            Self::SelectedCompany(i) => {
                let selected_company = &model.selected_company;

                match selected_company.read() {
                    Some(old_one) => {
                        if old_one == i {
                            selected_company.put(None)
                        } else {
                            selected_company.put(Some(i))
                        }
                    }
                    None => selected_company.put(Some(i)),
                }
            }
            Self::SelectedCompanyBranch(i) => {
                model.selected_company_branch.put(Some(i));
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu:       false,
                    page_to_present: ui_model::Menu::Dashboard,
                }))
            }
        }
    }
}

fn spawn_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let data = wrap_input(Type1 {
        user_uuid: model.user_uuid.read().clone().unwrap(),
    });

    let listener_aborter = client_traits::spawn_listener::<Rn, Rt, Mpsc>(
        cache,
        LISTEN_TO_OPERATIONS,
        data,
        move |data| {
            let data = unwrap_output(data);
            apply_on_the_model(&data, model);
        },
    );

    commander_local_state.aborter_to_company_and_branch_listener.set(Box::new(listener_aborter));
}
