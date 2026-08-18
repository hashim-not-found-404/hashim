use crate::accounting_domain::cases;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::DBTransaction;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits;

impl cases::sign_up::Input {
    pub(crate) async fn handle_operation<
        Id: types::RowId,
        Auth: types::HashedPassword,
        Jwt: types::JWT,
        Cli: DBClient,
        Db: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Cli::Txn<'a>>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<cases::sign_up::MyResult, traits::DynamicError> {
        let errr = self.state_less_check::<Id>();
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let errr = self.state_full_check::<Db>(&mut txn).await?;
        if errr.is_there_error() {
            txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }
        let result = self.state_full_operation::<Auth, Jwt>(jwt);
        txn.write_sign_up(&result).await?;
        let _ = txn.commit_transaction().await?;

        side_effects.authenticated_users.insert(self.new_uuid.clone());

        Ok(Ok(result))
    }
}
