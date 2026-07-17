use crate::{
    accounting_domain::cases::{
        self,
        utility::types::{self, MyErrorTrait},
    },
    server::use_cases::utility::server_traits::{DBClient, DBTransaction, SideEffects},
    utility::traits,
};

impl cases::create_company_branch::Input {
    pub(crate) async fn handle_operation<Id: types::RowId, Cli: DBClient>(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
    ) -> Result<
        Result<cases::create_company_branch::Ok, cases::create_company_branch::Error>,
        traits::DynamicError,
    > {
        let mut errr = self.state_less_check::<Id>();
        if !side_effects.authenticated_users.contains(&self.user_uuid) {
            errr.user_uuid = Some(types::UserUuidError::NotAuthenticated);
        }
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let (user_roles, is_new_uuid_used, is_company_belong_exist, is_branch_name_used) = txn
            .read_create_company_branch(
                &self.new_uuid,
                &self.user_uuid,
                &self.company_belong,
                &self.branch_name,
            )
            .await?;

        let errr = self.state_full_check::<Id>(
            &user_roles,
            is_new_uuid_used,
            is_company_belong_exist,
            is_branch_name_used,
        );
        if errr.is_there_error() {
            let _ = txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }

        let result = self.state_less_operation();
        txn.write_create_company_branch(&result).await?;
        let _ = txn.commit_transaction().await?;

        return Ok(Ok(result));
    }
}
