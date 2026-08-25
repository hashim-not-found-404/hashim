use crate::domain::use_cases;
use crate::domain::utility::types::DatabaseWrite;
use crate::domain::utility::types::HashedPassword;
use crate::domain::utility::types::JWT;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::DBTransaction;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits::DynamicError;

impl use_cases::sign_up::Input {
    pub(crate) async fn handle_operation<
        Id: RowId,
        Auth: HashedPassword,
        Jwt: JWT,
        Cli: DBClient,
        Db: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Cli::Txn<'a>>,
        DbWrite: for<'a> DatabaseWrite<Db<'a> = Cli::Txn<'a>, Input = use_cases::sign_up::Ok>,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<use_cases::sign_up::MyResult, DynamicError> {
        let errr = self.state_less_check::<Id>();
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;

        let result = async {
            let errr = self.state_full_check::<Db>(&mut txn).await?;

            if errr.is_there_error() {
                return Ok(Err(errr));
            }

            let result = self.state_full_operation::<Auth, Jwt>(jwt);
            DbWrite::write(&mut txn, &result).await?;
            side_effects.authenticated_users.insert(self.user_uuid.clone());
            Ok(Ok(result))
        }
        .await;

        if let Ok(Ok(_)) = result {
            let _ = txn.commit_transaction().await?;
        } else {
            txn.rollback_transaction().await?;
        }

        result
    }
}
