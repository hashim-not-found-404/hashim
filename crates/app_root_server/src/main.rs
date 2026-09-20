use database::db_client;
use kernel::server::CastDTOToServer;
use kernel::server::OperationsInputServer;
use server::app;
use std::any::Any;
use utility::dtos::TypeOperationsInput;

macro_rules! downcast {
    ($v:expr, $crate_name:tt) => {
        if let Some(v) = $v.downcast_ref::<$crate_name::domain::Input>() {
            return Box::new(v.clone());
        };
    };
}

struct Cas;

impl CastDTOToServer for Cas {
    type Cli = db_client::S;

    fn cast_input(v: TypeOperationsInput) -> Box<dyn OperationsInputServer<Cli = Self::Cli>> {
        let v: Box<dyn Any> = v;

        downcast!(v, use_case_get_all_accounts);
        downcast!(v, use_case_create_account);

        unreachable!()
    }
}

#[actix_web::main]
async fn main() {
    app::main::<Cas>().await;
}
