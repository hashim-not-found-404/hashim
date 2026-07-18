use crate::{
    accounting_domain::cases::{
        self,
        utility::types::{self, MyErrorTrait},
    },
    server::use_cases::utility::server_traits::{DBClient, DBTransaction, SideEffects},
    utility::traits,
};

impl cases::create_account::Input {
    pub(crate) async fn handle_operation<Id: types::RowId, Cli: DBClient>(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<cases::create_account::MyResult, traits::DynamicError> {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let read_output = txn
            .read_create_account(&cases::create_account::ReadInput {
                user_uuid: self.user_uuid.clone(),
                new_uuid: self.new_uuid.clone(),
                belong_to_company: self.belong_to_company.clone(),
                account_name: self.account_name.clone(),
            })
            .await?;

        let errr = self.state_full_check(&read_output);
        if errr.is_there_error() {
            let _ = txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }

        let result = self.state_less_operation();
        txn.write_create_account(&result).await?;
        let _ = txn.commit_transaction().await?;

        return Ok(Ok(result));
    }
}
