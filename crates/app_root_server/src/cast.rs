use database::db_client;
use kernel::server::CastDTOToServer;
use kernel::server::TraitOperationServerInput;
use paste::paste;
use std::any::Any;
use utility::dtos::TypeOperationDTOInput;

macro_rules! downcast {
    ($v:expr, $crate_name:tt) => {
        if let Some(v) = $v.downcast_ref::<$crate_name::domain::Input>() {
            paste! {
                return Box::new(crate::wire::[<Wrapper $crate_name:camel>](v.clone()));
            };
        };
    };
}

pub(crate) struct MyCaster;

impl CastDTOToServer for MyCaster {
    type Cli = db_client::S;

    fn cast_input(v: TypeOperationDTOInput) -> Box<dyn TraitOperationServerInput<Cli = Self::Cli>> {
        let v: Box<dyn Any> = v;

        downcast!(v, use_case_get_all_accounts);
        downcast!(v, use_case_create_account);

        unreachable!()
    }
}
