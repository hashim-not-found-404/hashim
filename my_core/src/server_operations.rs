use crate::prelude::*;
use crate::server_methods::Resource;
use std::result::Result as StdResult;

pub(crate) trait ServerOperations {
    type Ok;
    type Error;

    async fn handle_operation<At: AllServerTypes>(
        &self,
        side_effects: &mut server_methods::SideEffects,
        client: &mut At::Cli,
        jwt: &At::Jwt,
    ) -> Result<Result<Self::Ok, Self::Error>, DynamicError>;
}

pub mod sign_up {
    use super::*;

    pub type Result = StdResult<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub new_uuid: db_types::UuidType,
        pub name: Option<String>,
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub resource: Vec<ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub new_uuid: Option<RowIdError>,
        pub user_id: Option<UserIdError>,
        pub name: Option<String>,
    }

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum UserIdError {
        Duplicated,
    }

    impl ServerOperations for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle_operation<At: AllServerTypes>(
            &self,
            side_effects: &mut server_methods::SideEffects,
            client: &mut At::Cli,
            jwt: &At::Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            match At::Id::validate(&self.new_uuid) {
                true => {
                    side_effects
                        .authenticated_users
                        .insert(self.new_uuid.clone());
                }
                false => {
                    errr.new_uuid = Some(RowIdError::Invalid);
                }
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let hashed_password = At::Auth::sign_up(&self.password);

            let mut txn = client.begin_transaction().await?;

            let result = (|| async {
                let (is_new_uuid_exist, is_user_id_exist) =
                    txn.read_sign_up(&self.new_uuid, &self.user_id).await?;

                if is_new_uuid_exist {
                    errr.new_uuid = Some(RowIdError::Duplicated);
                }

                if is_user_id_exist {
                    errr.user_id = Some(UserIdError::Duplicated);
                }

                if errr != Self::Error::default() {
                    return Ok(Err(errr));
                }

                txn.write_sign_up(&self.new_uuid, &self.user_id, &hashed_password, &self.name)
                    .await?;

                let mut resource = Vec::new();

                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::Jwt(jwt.sign(&self.new_uuid)),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableUserFieldId(self.user_id.clone()),
                });
                if let Some(name) = self.name.clone() {
                    resource.push(ResourceInfo {
                        row_uuid: self.new_uuid.clone(),
                        resource: Resource::TableUserFieldName(name),
                    });
                }

                Ok(Ok(Self::Ok { resource }))
            })()
            .await;

            if let Ok(Ok(_)) = &result {
                let _ = txn.commit_transaction().await?;
            } else {
                let _ = txn.rollback_transaction().await?;
            }

            return result;
        }
    }
}

pub mod sign_in {
    use super::*;

    pub type Result = StdResult<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub resource: Vec<ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_id: Option<UserIdError>,
        pub password: Option<PasswordError>,
    }

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum UserIdError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum PasswordError {
        WrongPassword,
    }

    impl ServerOperations for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle_operation<At: AllServerTypes>(
            &self,
            side_effects: &mut server_methods::SideEffects,
            client: &mut At::Cli,
            jwt: &At::Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            let (user_rowid, password_hash) = match client.read_sign_in(&self.user_id).await? {
                Some(p) => p,
                None => {
                    errr.user_id = Some(UserIdError::NotExist);
                    return Ok(Err(errr));
                }
            };

            match At::Auth::sign_in(&self.password, &password_hash) {
                true => {
                    side_effects.authenticated_users.insert(user_rowid.clone());
                    side_effects.users_to_resubscribe.insert(user_rowid.clone());

                    let mut resource = Vec::new();

                    resource.push(ResourceInfo {
                        row_uuid: user_rowid.clone(),
                        resource: Resource::Jwt(jwt.sign(&user_rowid)),
                    });

                    return Ok(Ok(Self::Ok { resource }));
                }
                false => {
                    errr.password = Some(PasswordError::WrongPassword);
                    return Ok(Err(errr));
                }
            };
        }
    }
}

pub mod create_company {
    use super::*;

    pub type Result = StdResult<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: db_types::UuidType,
        pub new_uuid: db_types::UuidType,
        pub company_name: String,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub resource: Vec<ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<UserUuidError>,
        pub new_uuid: Option<RowIdError>,
    }

    impl ServerOperations for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle_operation<At: AllServerTypes>(
            &self,
            side_effects: &mut server_methods::SideEffects,
            client: &mut At::Cli,
            jwt: &At::Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            if !At::Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(RowIdError::Invalid);
            }

            match At::Id::validate(&self.user_uuid) {
                true => {
                    if side_effects
                        .authenticated_users
                        .get(&self.user_uuid)
                        .is_none()
                    {
                        errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                    };
                }
                false => {
                    errr.user_uuid = Some(UserUuidError::Invalid);
                }
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let mut txn = client.begin_transaction().await?;

            let result = (|| async {
                let is_new_uuid_used = txn.read_create_company(&self.new_uuid).await?;

                if is_new_uuid_used {
                    errr.new_uuid = Some(RowIdError::Duplicated);
                    return Ok(Err(errr));
                }

                const ROLE: db_types::Role = db_types::Role::Manager;

                txn.write_create_company(
                    &self.new_uuid,
                    &self.user_uuid,
                    &ROLE,
                    &self.company_name,
                    &self.currency,
                )
                .await?;

                side_effects
                    .users_to_resubscribe
                    .insert(self.user_uuid.clone());

                let v = ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: server_methods::Resource::TableCompanyFieldName(
                        self.company_name.clone(),
                    ),
                };
                let v1 = ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: server_methods::Resource::TableCompanyFieldCurrency(
                        self.currency.clone(),
                    ),
                };
                let v2 = ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(ROLE),
                };
                let v3 = ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: server_methods::Resource::TableAccessControlForCompanyFieldUser(
                        self.user_uuid.clone(),
                    ),
                };
                let v4 = ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(
                        self.new_uuid.clone(),
                    ),
                };

                side_effects
                    .resource_to_broadcast_for_company
                    .insert_push(self.new_uuid.clone(), v.clone());
                side_effects
                    .resource_to_broadcast_for_company
                    .insert_push(self.new_uuid.clone(), v1.clone());
                side_effects
                    .resource_to_broadcast_for_company
                    .insert_push(self.new_uuid.clone(), v2.clone());
                side_effects
                    .resource_to_broadcast_for_company
                    .insert_push(self.new_uuid.clone(), v3.clone());
                side_effects
                    .resource_to_broadcast_for_company
                    .insert_push(self.new_uuid.clone(), v4.clone());

                Ok(Ok(Self::Ok {
                    resource: vec![v, v1, v2, v3, v4],
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
    }
}

pub mod list_company_and_branch {
    use super::*;

    pub type Result = StdResult<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: db_types::UuidType,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub resource: Vec<ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<UserUuidError>,
    }

    impl ServerOperations for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle_operation<At: AllServerTypes>(
            &self,
            side_effects: &mut server_methods::SideEffects,
            client: &mut At::Cli,
            jwt: &At::Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            match At::Id::validate(&self.user_uuid) {
                true => {
                    if side_effects
                        .authenticated_users
                        .get(&self.user_uuid)
                        .is_none()
                    {
                        errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                    };
                }
                false => {
                    errr.user_uuid = Some(UserUuidError::Invalid);
                }
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let resource = client.read_list_company_and_branch(&self.user_uuid).await?;

            Ok(Ok(Self::Ok { resource }))
        }
    }
}

pub mod create_company_branch {
    use super::*;

    pub type Result = StdResult<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: db_types::UuidType,
        pub new_uuid: db_types::UuidType,
        pub company_belong: db_types::UuidType,
        pub branch_name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub resource: Vec<ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<UserUuidError>,
        pub new_uuid: Option<RowIdError>,
        pub company_belong: Option<CompanyBelongError>,
        pub branch_name: Option<BranchNameError>,
        pub location: Option<LocationError>,
    }

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum CompanyBelongError {
        IdInWrongFormat,
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum BranchNameError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum LocationError {
        Invalid,
    }

    impl ServerOperations for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle_operation<At: AllServerTypes>(
            &self,
            side_effects: &mut server_methods::SideEffects,
            client: &mut At::Cli,
            jwt: &At::Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            if !At::Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(RowIdError::Invalid);
            }

            match At::Id::validate(&self.user_uuid) {
                true => {
                    if side_effects
                        .authenticated_users
                        .get(&self.user_uuid)
                        .is_none()
                    {
                        errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                    };
                }
                false => {
                    errr.user_uuid = Some(UserUuidError::Invalid);
                }
            };

            if !At::Id::validate(&self.company_belong) {
                errr.company_belong = Some(CompanyBelongError::IdInWrongFormat);
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let mut txn = client.begin_transaction().await?;

            let result = (|| async {
                let (user_roles, is_new_uuid_used, is_company_exist, is_branch_name_used) = txn
                    .read_create_company_branch(
                        &self.new_uuid,
                        &self.user_uuid,
                        &self.company_belong,
                        &self.branch_name,
                    )
                    .await?;

                if !db_types::Role::has_any(
                    &user_roles,
                    &[db_types::Role::Manager, db_types::Role::CoManager],
                ) {
                    errr.user_uuid = Some(UserUuidError::YouDontHavePermissionToDoThat);
                }

                if is_new_uuid_used {
                    errr.new_uuid = Some(RowIdError::Duplicated);
                }

                if !is_company_exist {
                    errr.company_belong = Some(CompanyBelongError::NotExist);
                }

                if is_branch_name_used {
                    errr.branch_name = Some(BranchNameError::Duplicated);
                }

                if !self.location.is_valid() {
                    errr.location = Some(LocationError::Invalid);
                }

                if errr != Self::Error::default() {
                    return Ok(Err(errr));
                }

                const ROLE: db_types::Role = db_types::Role::CoManager;

                txn.write_create_company_branch(
                    &self.new_uuid,
                    &self.company_belong,
                    &self.branch_name,
                    &self.location,
                    &self.currency,
                    &self.user_uuid,
                    &ROLE,
                )
                .await?;

                side_effects
                    .users_to_resubscribe
                    .insert(self.user_uuid.clone());

                let mut resource = Vec::new();

                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldName(self.branch_name.clone()),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldCompanyBelong(
                        self.company_belong.clone(),
                    ),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                        self.new_uuid.clone(),
                    ),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyBranchFieldRole(ROLE),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableAccessControlForCompanyBranchFieldUser(
                        self.user_uuid.clone(),
                    ),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldCurrency(self.currency.clone()),
                });
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableCompanyBranchFieldLocation(self.location.clone()),
                });

                side_effects
                    .resource_to_broadcast_for_company
                    .insert_append(self.new_uuid.clone(), resource.clone());

                Ok(Ok(Self::Ok { resource }))
            })()
            .await;

            if let Ok(Ok(_)) = &result {
                let _ = txn.commit_transaction().await?;
            } else {
                let _ = txn.rollback_transaction().await?;
            }

            return result;
        }
    }
}
