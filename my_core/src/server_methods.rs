use crate::prelude::*;

pub struct ServerMethods<DB, Cli, Jwt, Authentication, F, Id>
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

impl<DB, Cli, Jwt, Authentication, F, Id> ServerMethods<DB, Cli, Jwt, Authentication, F, Id>
where
    DB: Database<Client = Cli>,
    Cli: DBClient<RowId = Id, HashedPassword = Authentication>,
    for<'a> Cli::Txn<'a>: DBTransaction<RowId = Id, HashedPassword = Authentication>,
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

    pub async fn sign_up(&self, input: &sign_up::Input) -> Result<sign_up::Result, DynamicError> {
        let hashed_password = Authentication::sign_up(&input.password);

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

    pub async fn sign_in(&self, input: &sign_in::Input) -> Result<sign_in::Result, DynamicError> {
        let mut errr = sign_in::Error {
            user_id: None,
            password: None,
        };

        let mut client = self.database.get_client().await?;

        let (user_rowid, password_hash) = match client.read_sign_in(&input.user_id).await? {
            Some(p) => p,
            None => {
                errr.user_id = Some(sign_in::UserIdError::NotExist);
                return Ok(Err(errr));
            }
        };

        match Authentication::sign_in(&input.password, &password_hash) {
            true => {
                return Ok(Ok(sign_in::Ok {
                    jwt: self.jwt.sign(&user_rowid).into(),
                }));
            }
            false => {
                errr.password = Some(sign_in::PasswordError::WrongPassword);
                return Ok(Err(errr));
            }
        };
    }
}
