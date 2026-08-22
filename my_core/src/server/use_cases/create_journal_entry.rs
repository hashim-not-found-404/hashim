use crate::domain::use_cases;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::types::UserUuidError;
use crate::server::utility::server_traits;
use crate::server::utility::server_traits::DBClient;
use crate::server::utility::server_traits::DBTransaction;
use crate::server::utility::server_traits::SideEffects;
use crate::utility::traits;
use crate::utility::traits::DynamicError;

impl use_cases::create_journal_entry::Input {
    pub(crate) async fn handle_operation<
        Id: RowId,
        Ti: traits::Time,
        Cli: DBClient,
        Db: for<'a> use_cases::create_journal_entry::DatabaseRead<Db<'a> = Cli::Txn<'a>>,
        DbWrite: for<'a> server_traits::DatabaseWrite<
                Db<'a> = Cli::Txn<'a>,
                Input = use_cases::create_journal_entry::Ok,
            >,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<use_cases::create_journal_entry::MyResult, DynamicError> {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;

        let result = async {
            let result = self.state_full_check::<Db, Ti>(&mut txn).await?;

            let Ok(result) = result else {
                return Ok(Err(errr));
            };

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
