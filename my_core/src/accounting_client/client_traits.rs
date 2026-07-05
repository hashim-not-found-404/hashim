use crate::{
    accounting_client::client_types,
    accounting_domain::{cases, request_response, types},
    utility::{traits, utils},
};

pub trait WSClient: Sized {
    fn connect(url: &str) -> impl Future<Output = Result<Self, utils::DynamicError>>;
    fn send_bin(&self, data: &Vec<u8>) -> impl Future<Output = Result<(), utils::DynamicError>>;
    fn receive_bin(&self) -> impl Future<Output = Result<Vec<u8>, utils::DynamicError>>;
}

pub trait Cache: Sized {
    fn new() -> impl Future<Output = Self>;

    fn get_all_txn_input(
        &self,
    ) -> impl Future<
        Output = Vec<
            request_response::push_data::Txn<request_response::push_data::OperationsInput>,
        >,
    >;
    fn write_txn_input(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsInput>,
    ) -> impl Future<Output = ()>;
    fn write_txn_result(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsResult>,
    ) -> impl Future<Output = ()>;
    fn mark_txn_input_as_faild(&self, txn_number: &u64) -> impl Future<Output = ()>;
    fn delete_txn_input(&self, txn_number: &u64) -> impl Future<Output = ()>;

    fn write_resource(&self, resource: &Vec<types::ResourceInfo>) -> impl Future<Output = ()>;
    fn get_jwt(
        &self,
        user_uuid: &types::UuidType,
    ) -> impl Future<Output = Option<types::JsonWebTokenType>>;

    fn read_sign_up(
        &self,
        new_uuid: &types::UuidType,
        user_id: &String,
    ) -> impl Future<
        Output = (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
    >;
    fn read_sign_in(
        &self,
        user_id: &String,
    ) -> impl Future<
        Output = Option<(
            types::UuidType, /* user uuid */
            Option<String>,  /* user name */
            bool,            /* does he have jwt */
        )>,
    >;
    fn read_list_company_and_branch(
        &self,
        user_uuid: &types::UuidType,
    ) -> impl Future<Output = Vec<types::ResourceInfo>>;
    fn read_create_company_branch(
        &self,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        company_branch_name: &String,
    ) -> impl Future<
        Output = (
            Vec<types::Role>, /* roles at company */
            bool,             /* is company exist */
            bool,             /* is branch name used */
        ),
    >;
}

pub trait HashimSignal<T: Default + Clone>: Default {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllClientTypes: 'static + Default + Clone {
    type Rn: traits::RandomNumber;
    type Rt: traits::Runtime;
    type Id: cases::RowId;
    type Mpsc: traits::MultiProducerSingleConsumer;
    type Ed: traits::Coding;
    type Rg: traits::Regex;

    type Ch: Cache;
    type Ws: WSClient;

    // signals
    type String: HashimSignal<String>;
    type Dialog: HashimSignal<client_types::Dialog>;
    type Uuid: HashimSignal<types::UuidType>;
    type OptionUuid: HashimSignal<Option<types::UuidType>>;
    type Bool: HashimSignal<bool>;
    type StringVec: HashimSignal<String>;
    type Currency: HashimSignal<types::Currency>;
    type Location: HashimSignal<types::Location>;
    type CompanyAndBranchList: HashimSignal<Vec<types::Company>>;

    type Navigator: HashimSignal<client_types::Navigator>;
}
