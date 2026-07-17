use crate::{
    accounting_domain::cases::{
        self,
        utility::types::{self, MyErrorTrait},
    },
    server::use_cases::utility::server_traits::{DBClient, DBTransaction, SideEffects},
    utility::traits,
};

impl cases::sign_up::Input {
    pub(crate) async fn handle_operation<
        Id: types::RowId,
        Auth: types::HashedPassword,
        Jwt: types::JWT,
        Cli: DBClient,
    >(
        &self,
        side_effects: &mut SideEffects,
        client: &mut Cli,
        jwt: &Jwt,
    ) -> Result<Result<cases::sign_up::Ok, cases::sign_up::Error>, traits::DynamicError> {
        let errr = self.state_less_check::<Id>();
        if errr.is_there_error() {
            return Ok(Err(errr));
        }

        let mut txn = client.begin_transaction().await?;
        let (is_new_uuid, is_user_id) = txn.read_sign_up(&self.new_uuid, &self.user_id).await?;

        let errr = self.state_full_check::<Id>(is_new_uuid, is_user_id);
        if errr.is_there_error() {
            let _ = txn.rollback_transaction().await?;
            return Ok(Err(errr));
        }
        let result = self.state_full_operation::<Auth, Jwt>(jwt);
        txn.write_sign_up(&result).await?;
        let _ = txn.commit_transaction().await?;

        side_effects
            .authenticated_users
            .insert(self.new_uuid.clone());

        return Ok(Ok(result));
    }
}
