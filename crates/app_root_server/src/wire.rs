use patterns::make_server_wrapper_read;
use patterns::make_server_wrapper_write;

make_server_wrapper_write!(create_account);
make_server_wrapper_read!(get_all_accounts);
make_server_wrapper_write!(sign_up);

use anyhow::Result;
use anyhow::bail;
use database::db_client;
use infrastructure::jwt::Jwt;
use kernel::server::CastDTOToServer;
use kernel::server::TraitOperationServerInput;
use paste::paste;
use std::any::Any;
use utility::dtos::TypeOperationDTOInput;

macro_rules! downcast {
    ($v:expr, $crate_name:tt) => {
        if let Some(v) = $v.downcast_ref::<$crate_name::domain::Input>() {
            paste! {
                return Ok(Box::new([<server_ $crate_name>]::Wrapper(v.clone())));
            };
        };
    };
}

pub(crate) struct MyCaster;

impl CastDTOToServer for MyCaster {
    type Cli = db_client::S;
    type Jwt = Jwt;

    fn cast_input(
        v: TypeOperationDTOInput,
    ) -> Result<Box<dyn TraitOperationServerInput<Cli = Self::Cli, Jwt = Self::Jwt>>> {
        let v: Box<dyn Any> = v;

        downcast!(v, use_case_get_all_accounts);
        downcast!(v, use_case_create_account);
        downcast!(v, use_case_sign_up);

        bail!("downcast error")
    }
}
