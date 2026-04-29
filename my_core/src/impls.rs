use crate::{db_types, request_response::*, traits::*};
use std::marker::PhantomData;

pub struct StateFullCheck<DB, Cli, Jwt, Authentication, F, Id>
where
    DB: Database<Client = Cli>,
    Cli: DBClient,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId,
{
    database: DB,
    client: PhantomData<Cli>,
    jwt: Jwt,
    authentication: PhantomData<Authentication>,
    functions: PhantomData<F>,
    rowid: PhantomData<Id>,
}

// TODO : this should implement trait BackendRouts
impl<DB, Cli, Jwt, Authentication, F, Id, ExternalError>
    StateFullCheck<DB, Cli, Jwt, Authentication, F, Id>
where
    DB: Database<Error = ExternalError, Client = Cli>,
    Cli: DBClient<Error = ExternalError>,
    for<'a> Cli::Txn<'a>:
        DBTransaction<Error = ExternalError, RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId,
{
    pub fn new(database: DB, jwt: Jwt) -> Self {
        Self {
            database,
            client: PhantomData::<Cli>,
            jwt,
            authentication: PhantomData::<Authentication>,
            functions: PhantomData::<F>,
            rowid: PhantomData::<Id>,
        }
    }

    // async fn sign_in(
    //     &self,
    //     txn: &mut Cli::Txn<'_>,
    //     input: sign_in::Input,
    // ) -> sign_in::Result<ExternalError> {
    //     let (user_rowid, password_hash) = match txn
    //         .select_user_rowid_and_password_hash(&input.user_id)
    //         .await?
    //     {
    //         Some(p) => p,
    //         None => {
    //             return Ok(Err(business_layer::Error::InvalidInput(sign_in::Error {
    //                 user_id: Some(sign_in::UserIdError::NotExist),
    //                 password: None,
    //             })));
    //         }
    //     };

    //     match Authentication::sign_in(input.password, password_hash) {
    //         true => {
    //             return Ok(Ok(sign_in::Ok {
    //                 jwt: self.jwt.sign(&user_rowid).into(),
    //             }));
    //         }
    //         false => {
    //             return Ok(Err(business_layer::Error::InvalidInput(sign_in::Error {
    //                 user_id: None,
    //                 password: Some(sign_in::PasswordError::WrongPassword),
    //             })));
    //         }
    //     };
    // }

    // async fn get_all_user_roles(
    //     &self,
    //     txn: &mut Cli::Txn<'_>,
    //     input: get_all_user_roles::Input,
    //     user_id: &Id,
    // ) -> get_all_user_roles::Result<ExternalError> {
    //     let r = txn
    //         .select_all_companies_and_branches_for_the_user(user_id)
    //         .await?;
    //     todo!()
    // }

    // async fn create_company(
    //     &self,
    //     txn: &mut Cli::Txn<'_>,
    //     input: create_company::Input,
    //     user_id: &Id,
    // ) -> create_company::Result<ExternalError> {
    //     let row_id = Id::generate();

    //     txn.insert_company(&row_id, &input.name, &input.currency)
    //         .await?;

    //     txn.insert_role(
    //         &row_id,
    //         &custom_types::Role::Manager,
    //         &db_types::DataGroup::Company(row_id.clone()),
    //         user_id,
    //     )
    //     .await?;

    //     Ok(Ok(create_company::Ok))
    // }

    // async fn create_company_branch(
    //     &self,
    //     txn: &mut Cli::Txn<'_>,
    //     input: create_company_branch::Input,
    //     user_id: &Id,
    // ) -> create_company_branch::Result<ExternalError> {
    //     let mut errr = create_company_branch::Error {
    //         company_belong: None,
    //         location: None,
    //         name: None,
    //     };

    //     // TODO check the name if duplicated , check id if not exist

    //     let row_id = Id::generate();
    //     // txn.insert_company_branch(&row_id, &a, &data.name, &data.location, &data.currency)
    //     //     .await;

    //     txn.insert_role(
    //         &row_id,
    //         &custom_types::Role::Manager,
    //         &db_types::DataGroup::Branch(user_id.clone()),
    //         user_id,
    //     )
    //     .await?;

    //     Ok(Ok(create_company_branch::Ok))
    // }
}

impl<DB, Cli, Jwt, Authentication, F, Id, ExternalError> BackendRouts
    for StateFullCheck<DB, Cli, Jwt, Authentication, F, Id>
where
    DB: Database<Error = ExternalError, Client = Cli>,
    Cli: DBClient<Error = ExternalError>,
    for<'a> Cli::Txn<'a>:
        DBTransaction<Error = ExternalError, RowId = Id, HashedPassword = Authentication>,
    Jwt: JWT<UserId = Id>,
    Authentication: HashedPassword,
    F: Functions,
    Id: RowId,
{
    type Error = ExternalError;

    async fn sign_up(&self, input: sign_up::Input) -> Result<sign_up::Result, ExternalError> {
        let hashed_password = Authentication::sign_up(input.password);

        let mut errr = sign_up::Error {
            user_id: None,
            name: None,
        };

        let mut client = self.database.get_client().await?;
        let mut txn = client.begin_transaction().await?;

        let result = (|| async {
            let is_new_user = txn.read_sign_up(&input.user_id).await?;

            if !is_new_user {
                errr.user_id = Some(sign_up::UserIdError::Duplicated);
                return Ok(Err(errr));
            }

            let user_uuid = txn
                .write_sign_up(&input.user_id, &hashed_password, &input.name)
                .await?;

            Ok(Ok(sign_up::Ok {
                jwt: self.jwt.sign(&user_uuid).into(),
            }))
        })()
        .await;

        if let Ok(Ok(_)) = &result {
            let _ = txn.commit_transaction().await?;
        } else {
            let _ = txn.rollback_transaction().await?;
        }

        return result;
    }

    //     async fn sign_in(&self, input: business_layer::Input) -> sign_in::Result<ExternalError> {
    //         let mut client = self.database.get_client().await?;

    //         for _ in 0..2_u8 {
    //             let mut txn = client.begin_transaction().await?;

    //             let is_ok = txn
    //                 .insert_transaction_if_new(input.transaction_number)
    //                 .await?;
    //             if !is_ok {
    //                 txn.rollback_transaction().await?;
    //                 return Ok(Err(business_layer::Error::DuplicateTransaction));
    //             }

    //             let result = self.sign_in(&mut txn, input.content.clone()).await;
    //             match result {
    //                 Ok(o) => {
    //                     if input.submit != business_layer::OperationMode::SubmitToServer {
    //                         txn.rollback_transaction().await?;
    //                         return Ok(o);
    //                     }
    //                     match txn.commit_transaction().await? {
    //                         Ok(_) => return Ok(o),
    //                         Err(domain_errors::AtCommit::DataIsChanged) => continue,
    //                     }
    //                 }
    //                 Err(e) => {
    //                     txn.rollback_transaction().await?;
    //                     return Err(e);
    //                 }
    //             }
    //         }

    //         return Ok(Err(business_layer::Error::DataHasBeenChangedByOthers));
    //     }

    //     async fn get_all_user_roles(
    //         &self,
    //         input: business_layer::Input,
    //     ) -> business_layer::Result<get_all_user_roles::Ok, get_all_user_roles::Error, Self::Error>
    //     {
    //         let mut client = self.database.get_client().await?;

    //         for _ in 0..2_u8 {
    //             let mut txn = client.begin_transaction().await?;

    //             let is_ok = txn
    //                 .insert_transaction_if_new(input.transaction_number)
    //                 .await?;
    //             if !is_ok {
    //                 txn.rollback_transaction().await?;
    //                 return Ok(Err(business_layer::Error::DuplicateTransaction));
    //             }

    //             let id = match self.jwt.validate(input.jwt.clone().into()) {
    //                 Ok(a) => a,
    //                 Err(_) => return Ok(Err(business_layer::Error::InvalidJWT)),
    //             };
    //             let result = self
    //                 .get_all_user_roles(&mut txn, input.content.clone(), &id)
    //                 .await;

    //             match result {
    //                 Ok(o) => {
    //                     if input.submit != business_layer::OperationMode::SubmitToServer {
    //                         txn.rollback_transaction().await?;
    //                         return Ok(o);
    //                     }
    //                     match txn.commit_transaction().await? {
    //                         Ok(_) => return Ok(o),
    //                         Err(domain_errors::AtCommit::DataIsChanged) => continue,
    //                     }
    //                 }
    //                 Err(e) => {
    //                     txn.rollback_transaction().await?;
    //                     return Err(e);
    //                 }
    //             }
    //         }

    //         return Ok(Err(business_layer::Error::DataHasBeenChangedByOthers));
    //     }

    //     async fn create_company(
    //         &self,
    //         input: business_layer::Input,
    //     ) -> business_layer::Result<create_company::Ok, create_company::Error, Self::Error> {
    //         let mut client = self.database.get_client().await?;

    //         for _ in 0..2_u8 {
    //             let mut txn = client.begin_transaction().await?;

    //             let is_ok = txn
    //                 .insert_transaction_if_new(input.transaction_number)
    //                 .await?;
    //             if !is_ok {
    //                 txn.rollback_transaction().await?;
    //                 return Ok(Err(business_layer::Error::DuplicateTransaction));
    //             }

    //             let id = match self.jwt.validate(input.jwt.clone().into()) {
    //                 Ok(a) => a,
    //                 Err(_) => return Ok(Err(business_layer::Error::InvalidJWT)),
    //             };
    //             let result = self
    //                 .create_company(&mut txn, input.content.clone(), &id)
    //                 .await;

    //             match result {
    //                 Ok(o) => {
    //                     if input.submit != business_layer::OperationMode::SubmitToServer {
    //                         txn.rollback_transaction().await?;
    //                         return Ok(o);
    //                     }
    //                     match txn.commit_transaction().await? {
    //                         Ok(_) => return Ok(o),
    //                         Err(domain_errors::AtCommit::DataIsChanged) => continue,
    //                     }
    //                 }
    //                 Err(e) => {
    //                     txn.rollback_transaction().await?;
    //                     return Err(e);
    //                 }
    //             }
    //         }

    //         return Ok(Err(business_layer::Error::DataHasBeenChangedByOthers));
    //     }

    //     async fn create_company_branch(
    //         &self,
    //         input: business_layer::Input,
    //     ) -> business_layer::Result<create_company_branch::Ok, create_company_branch::Error, Self::Error>
    //     {
    //         todo!()
    //     }
}

// pub struct StateLessCheck<SFC, Id>
// where
//     SFC: BackendRouts,
//     Id: RowId,
// {
//     state_full_check: SFC,
//     rowid: PhantomData<Id>,
// }

// impl<SFC, Id> StateLessCheck<SFC, Id>
// where
//     SFC: BackendRouts,
//     Id: RowId,
// {
//     pub fn new(state_full_check: SFC) -> Self {
//         Self {
//             state_full_check,
//             rowid: PhantomData::<Id>,
//         }
//     }
// }

// impl<SFC, Id> BackendRouts for StateLessCheck<SFC, Id>
// where
//     SFC: BackendRouts,
//     Id: RowId,
// {
//     type Error = SFC::Error;

//     async fn sign_up(
//         &self,
//         input: business_layer::Input,
//     ) -> business_layer::Result<sign_up::Ok, sign_up::Error, Self::Error> {
//         self.state_full_check.sign_up(input).await
//     }

//     async fn sign_in(
//         &self,
//         input: business_layer::Input,
//     ) -> business_layer::Result<sign_in::Ok, sign_in::Error, Self::Error> {
//         self.state_full_check.sign_in(input).await
//     }

//     async fn get_all_user_roles(
//         &self,
//         input: business_layer::Input,
//     ) -> business_layer::Result<get_all_user_roles::Ok, get_all_user_roles::Error, Self::Error>
//     {
//         self.state_full_check.get_all_user_roles(input).await
//     }

//     async fn create_company(
//         &self,
//         input: business_layer::Input,
//     ) -> business_layer::Result<create_company::Ok, create_company::Error, Self::Error> {
//         self.state_full_check.create_company(input).await
//     }

//     async fn create_company_branch(
//         &self,
//         input: business_layer::Input,
//     ) -> business_layer::Result<create_company_branch::Ok, create_company_branch::Error, Self::Error>
//     {
//         let mut errr = create_company_branch::Error {
//             company_belong: None,
//             location: None,
//             name: None,
//         };

//         let a = Id::try_from(input.content.company_belong.clone());
//         if a.is_err() {
//             errr.company_belong = Some(create_company_branch::CompanyError::IdInWrongFormat);
//         }

//         // TODO: check location if correct
//         self.state_full_check.create_company_branch(input).await
//     }
// }
