use crate::domain::use_cases;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::types::UserUuidError;
use crate::server::utility::server_traits;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::DBTransaction;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits::DynamicError;

impl use_cases::create_company::Input {
    pub(crate) async fn handle_operation<
        Id: RowId,
        Cli: DBClient,
        Db: for<'a> use_cases::create_company::DatabaseRead<Db<'a> = Cli::Txn<'a>>,
        DbWrite: for<'a> server_traits::DatabaseWrite<
                Db<'a> = Cli::Txn<'a>,
                Input = use_cases::create_company::Ok,
            >,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<use_cases::create_company::MyResult, DynamicError> {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;

        let result = async {
            let errr = self.state_full_check::<Db>(&mut txn).await?;

            if errr.is_there_error() {
                return Ok(Err(errr));
            }

            let result = self.state_less_operation();
            DbWrite::write(&mut txn, &result).await?;
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
