use crate::domain::use_cases;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::types::UserUuidError;
use crate::make_auth_check;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits::DynamicError;

impl use_cases::list_company_and_branch::Input {
    pub(crate) async fn handle_operation<
        Id: RowId,
        Cli: DBClient,
        Db: for<'a> use_cases::list_company_and_branch::DatabaseRead<Db<'a> = Cli>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<
        Result<use_cases::list_company_and_branch::Ok, use_cases::list_company_and_branch::Error>,
        DynamicError,
    > {
        let mut errr = self.state_less_check::<Id>();
        make_auth_check!(side_effects, self, errr);
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let result = self.state_full_operation::<Db>(client).await?;

        Ok(Ok(result))
    }
}
