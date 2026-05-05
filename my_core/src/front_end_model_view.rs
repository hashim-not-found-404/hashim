use crate::prelude::*;

pub trait Signal: Default {
    type T;
    fn read(&self) -> Self::T;
    fn set(&self, v: Self::T);
}

pub struct State<
    WS: WebSocket,
    TxnNumGen: RandomNumber,
    StringSignal: Signal<T = String>,
    BoolSignal: Signal<T = bool>,
    ExternalErrorSignal: Signal<T = String>,
    CurrencySignal: Signal<T = db_types::Currency>,
    UserRolesSignal: Signal<T = Vec<db_types::Company>>,
> {
    // here for the app logic
    generate_transaction_number: PhantomData<TxnNumGen>,
    routs: client::RoutsForClientSide<WS>,
    jwt: Mutex<Option<String>>,

    // here every field to display
    pub is_signed_in: BoolSignal,
    pub is_loading_sign_in_or_up: BoolSignal,
    pub user_name: StringSignal,
    pub user_id: StringSignal,
    pub password: StringSignal,
    pub sign_in_error_for_user_id: StringSignal,
    pub sign_in_error_for_password: StringSignal,
    pub sign_up_error_for_user_id: StringSignal,
    pub sign_up_error_for_user_name: StringSignal,
    pub external_errors: ExternalErrorSignal,
    pub company_name: StringSignal,
    pub company_currency: CurrencySignal,
    pub all_user_roles: UserRolesSignal,
}

impl<
    WS: WebSocket,
    TxnNumGen: RandomNumber,
    StringSignal: Signal<T = String>,
    BoolSignal: Signal<T = bool>,
    ExternalErrorSignal: Signal<T = String>,
    CurrencySignal: Signal<T = db_types::Currency>,
    UserRolesSignal: Signal<T = Vec<db_types::Company>>,
>
    State<
        WS,
        TxnNumGen,
        StringSignal,
        BoolSignal,
        ExternalErrorSignal,
        CurrencySignal,
        UserRolesSignal,
    >
{
    pub fn new(routs: client::RoutsForClientSide<WS>) -> Self {
        Self {
            generate_transaction_number: PhantomData::<TxnNumGen>,
            routs: routs,
            jwt: Mutex::new(None),
            is_signed_in: BoolSignal::default(),
            is_loading_sign_in_or_up: BoolSignal::default(),
            user_name: StringSignal::default(),
            user_id: StringSignal::default(),
            password: StringSignal::default(),
            sign_in_error_for_user_id: StringSignal::default(),
            sign_in_error_for_password: StringSignal::default(),
            sign_up_error_for_user_id: StringSignal::default(),
            sign_up_error_for_user_name: StringSignal::default(),
            external_errors: ExternalErrorSignal::default(),
            company_name: StringSignal::default(),
            company_currency: CurrencySignal::default(),
            all_user_roles: UserRolesSignal::default(),
        }
    }

    pub async fn sign_up(&self) {
        if self.is_loading_sign_in_or_up.read() == true {
            return;
        }
        self.is_loading_sign_in_or_up.set(true);

        self.sign_in_error_for_user_id.set(String::new());
        self.sign_in_error_for_password.set(String::new());
        self.sign_up_error_for_user_id.set(String::new());
        self.sign_up_error_for_user_name.set(String::new());

        let input = sign_up::Input {
            name: {
                let x = self.user_name.read();
                if x.as_str() == "" {
                    None
                } else {
                    Some(x.to_string())
                }
            },
            user_id: self.user_id.read().to_string(),
            password: self.password.read().to_string(),
        };

        let result = self.routs.sign_up(&input).await;

        match result {
            Ok(Ok(business_output)) => {
                self.is_signed_in.set(true);
                *self.jwt.lock().unwrap() = Some(business_output.jwt.clone());
            }
            Ok(Err(business_error)) => {
                self.sign_up_error_for_user_id
                    .set(match business_error.user_id {
                        Some(_) => String::from("duplicated user"),
                        None => String::new(),
                    });
                self.sign_up_error_for_user_name
                    .set(match business_error.name {
                        Some(e) => e,
                        None => String::new(),
                    });
            }
            Err(external_error) => {
                self.external_errors.set(external_error.to_string());
            }
        }
        self.is_loading_sign_in_or_up.set(false);
    }

    pub async fn sign_in(&self) {
        if self.is_loading_sign_in_or_up.read() == true {
            return;
        }
        self.is_loading_sign_in_or_up.set(true);

        self.sign_in_error_for_user_id.set(String::new());
        self.sign_in_error_for_password.set(String::new());

        let input = sign_in::Input {
            user_id: self.user_id.read(),
            password: self.password.read(),
        };

        let result = self.routs.sign_in(&input).await;

        match result {
            Ok(Ok(business_output)) => {
                self.is_signed_in.set(true);
                *self.jwt.lock().unwrap() = Some(business_output.jwt.clone());
            }
            Ok(Err(business_error)) => {
                self.sign_in_error_for_user_id
                    .set(match business_error.user_id {
                        Some(_) => String::from("user not exist"),
                        None => String::new(),
                    });
                self.sign_in_error_for_password
                    .set(match business_error.password {
                        Some(_) => String::from("wrong password"),
                        None => String::new(),
                    });
            }
            Err(external_error) => {
                self.external_errors.set(external_error.to_string());
            }
        }

        self.is_loading_sign_in_or_up.set(false);
    }

    // pub async fn get_all_user_roles(&self) {
    //     let input = self.generate_txn(get_all_user_roles::Input {});

    //     let result = self.routs.get_all_user_roles(input).await;

    //     match result {
    //         Ok(Ok(business_output)) => {
    //             self.all_user_roles.set(business_output.all_roles);
    //         }
    //         Ok(Err(business_error)) => match business_error {
    //             business_layer::Error::InvalidInput(err) => {}
    //             business_layer::Error::InvalidJWT => self.is_signed_in.set(false),
    //             _ => todo!(),
    //         },
    //         Err(external_error) => {
    //             self.external_errors.set(external_error.to_string());
    //         }
    //     }
    // }

    // pub async fn create_company(&self) {
    //     let input = self.generate_txn(create_company::Input {
    //         name: self.company_name.read(),
    //         currency: self.company_currency.read(),
    //     });

    //     let result = self.routs.create_company(input).await;

    //     match result {
    //         Ok(Ok(business_output)) => {}
    //         Ok(Err(business_error)) => match business_error {
    //             business_layer::Error::InvalidInput(err) => {}
    //             business_layer::Error::InvalidJWT => self.is_signed_in.set(false),
    //             _ => todo!(),
    //         },
    //         Err(external_error) => {
    //             self.external_errors.set(external_error.to_string());
    //         }
    //     }
    // }
}
