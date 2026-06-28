use crate::prelude::*;
use crate::server_methods::Resource;
use std::result::Result as StdResult;

pub(crate) trait StateOp {
    async fn read_sign_up(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
    ) -> Result<
        (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
        DynamicError,
    >;

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(db_types::UuidType, String)>, DynamicError>;

    async fn read_create_company(
        &mut self,
        new_uuid: &db_types::UuidType,
    ) -> Result<bool /* is new_uuid exist */, DynamicError>;

    async fn read_list_company_and_branch(
        &mut self,
        user_uuid: &db_types::UuidType,
    ) -> Result<Vec<ResourceInfo>, DynamicError>;

    async fn read_create_company_branch(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        branch_name: &String,
    ) -> Result<
        (
            Vec<db_types::Role>, /* user roles */
            bool,                /* is new_uuid exist */
            bool,                /* is company_belong exist */
            bool,                /* is branch_name used */
        ),
        DynamicError,
    >;
}

pub(crate) trait EventMaker {
    /*
    i think insted of store resource and return resource is to return and store struct but the resource we need it to the broadcast , and that is better to prevent sql inject , and also the write on the server will e optimized

    The Trade‑Offs
    Aspect	        Vec<ResourceInfo>	Struct + Prepared Statement
    Flexibility     High	            ❌ Low (fixed schema)
    SQL Injection   ❌ Vulnerable        Safe
    Performance     ❌ N statements      1 statement
    Type Safety     ❌ Loose             Compiler‑checked
    Boilerplate     Low                 ❌ More code
     */
    type Ok: Into<Vec<ResourceInfo>>;
    type Error;

    async fn handle<
        St: StateOp,
        Rn: RandomNumber,
        Rt: Runtime,
        Id: RowId,
        Mpsc: MultiProducerSingleConsumer,
        Ed: Coding,
        Rg: Regex,
        Auth: HashedPassword,
        Jwt: JWT,
    >(
        &self,
        side_effects: &mut server_methods::SideEffects,
        state: &mut St,
        jwt: &Jwt,
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
        pub new_uuid: db_types::UuidType,
        pub user_id: String,
        pub user_name: Option<String>,
        pub hashed_password: String,
        jwt: db_types::JsonWebTokenType,
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

    impl Into<Vec<ResourceInfo>> for Ok {
        fn into(self) -> Vec<ResourceInfo> {
            let mut resource = Vec::new();

            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: Resource::Jwt(self.jwt),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: Resource::TableUserFieldId(self.user_id.clone()),
            });
            if let Some(name) = self.user_name.clone() {
                resource.push(ResourceInfo {
                    row_uuid: self.new_uuid.clone(),
                    resource: Resource::TableUserFieldName(name),
                });
            }

            resource
        }
    }

    impl EventMaker for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle<
            St: StateOp,
            Rn: RandomNumber,
            Rt: Runtime,
            Id: RowId,
            Mpsc: MultiProducerSingleConsumer,
            Ed: Coding,
            Rg: Regex,
            Auth: HashedPassword,
            Jwt: JWT,
        >(
            &self,
            side_effects: &mut server_methods::SideEffects,
            state: &mut St,
            jwt: &Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            if Id::validate(&self.new_uuid) {
                side_effects
                    .authenticated_users
                    .insert(self.new_uuid.clone());
            } else {
                errr.new_uuid = Some(RowIdError::Invalid);
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let hashed_password = Auth::sign_up(&self.password);

            let (is_new_uuid_exist, is_user_id_exist) =
                state.read_sign_up(&self.new_uuid, &self.user_id).await?;

            if is_new_uuid_exist {
                errr.new_uuid = Some(RowIdError::Duplicated);
            }

            if is_user_id_exist {
                errr.user_id = Some(UserIdError::Duplicated);
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let jwt = jwt.sign(&self.new_uuid);

            return Ok(Ok(Ok {
                new_uuid: self.new_uuid.clone(),
                user_id: self.user_id.clone(),
                user_name: self.name.clone(),
                hashed_password,
                jwt,
            }));
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
        pub user_uuid: db_types::UuidType,
        pub jwt: db_types::JsonWebTokenType,
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

    impl Into<Vec<ResourceInfo>> for Ok {
        fn into(self) -> Vec<ResourceInfo> {
            let mut resource = Vec::new();

            resource.push(ResourceInfo {
                row_uuid: self.user_uuid.clone(),
                resource: Resource::Jwt(self.jwt),
            });

            resource
        }
    }

    impl EventMaker for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle<
            St: StateOp,
            Rn: RandomNumber,
            Rt: Runtime,
            Id: RowId,
            Mpsc: MultiProducerSingleConsumer,
            Ed: Coding,
            Rg: Regex,
            Auth: HashedPassword,
            Jwt: JWT,
        >(
            &self,
            side_effects: &mut server_methods::SideEffects,
            state: &mut St,
            jwt: &Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            let (user_rowid, password_hash) = match state.read_sign_in(&self.user_id).await? {
                Some(p) => p,
                None => {
                    errr.user_id = Some(UserIdError::NotExist);
                    return Ok(Err(errr));
                }
            };

            match Auth::sign_in(&self.password, &password_hash) {
                true => {
                    side_effects.authenticated_users.insert(user_rowid.clone());
                    side_effects.users_to_resubscribe.insert(user_rowid.clone());

                    return Ok(Ok(Ok {
                        user_uuid: user_rowid.clone(),
                        jwt: jwt.sign(&user_rowid),
                    }));
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
        pub new_uuid: db_types::UuidType,
        pub company_name: String,
        pub currency: db_types::Currency,
        pub user_uuid: db_types::UuidType,
        pub role: db_types::Role,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<UserUuidError>,
        pub new_uuid: Option<RowIdError>,
    }

    impl Into<Vec<ResourceInfo>> for Ok {
        fn into(self) -> Vec<ResourceInfo> {
            let mut resource = Vec::new();

            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableCompanyFieldName(
                    self.company_name.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableCompanyFieldCurrency(
                    self.currency.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(
                    self.role.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableAccessControlForCompanyFieldUser(
                    self.user_uuid.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(
                    self.new_uuid.clone(),
                ),
            });

            resource
        }
    }

    impl EventMaker for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle<
            St: StateOp,
            Rn: RandomNumber,
            Rt: Runtime,
            Id: RowId,
            Mpsc: MultiProducerSingleConsumer,
            Ed: Coding,
            Rg: Regex,
            Auth: HashedPassword,
            Jwt: JWT,
        >(
            &self,
            side_effects: &mut server_methods::SideEffects,
            state: &mut St,
            jwt: &Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(RowIdError::Invalid);
            }

            if !Id::validate(&self.user_uuid) {
                errr.user_uuid = Some(UserUuidError::Invalid);
            }

            // TODO here is the bug
            if side_effects
                .authenticated_users
                .get(&self.user_uuid)
                .is_none()
            {
                errr.user_uuid = Some(UserUuidError::NotAuthenticated);
            };

            if errr != Self::Error::default() {
                mbg!(&errr);
                return Ok(Err(errr));
            }

            let is_new_uuid_used = state.read_create_company(&self.new_uuid).await?;

            if is_new_uuid_used {
                errr.new_uuid = Some(RowIdError::Duplicated);
                return Ok(Err(errr));
            }

            const ROLE: db_types::Role = db_types::Role::Manager;

            side_effects
                .users_to_resubscribe
                .insert(self.user_uuid.clone());

            let mut resource = Vec::new();

            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableCompanyFieldName(
                    self.company_name.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableCompanyFieldCurrency(
                    self.currency.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(ROLE),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableAccessControlForCompanyFieldUser(
                    self.user_uuid.clone(),
                ),
            });
            resource.push(ResourceInfo {
                row_uuid: self.new_uuid.clone(),
                resource: server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(
                    self.new_uuid.clone(),
                ),
            });

            side_effects
                .resource_to_broadcast_for_company
                .insert_append(self.new_uuid.clone(), resource.clone());

            Ok(Ok(Ok {
                new_uuid: self.new_uuid.clone(),
                company_name: self.company_name.clone(),
                currency: self.currency.clone(),
                user_uuid: self.user_uuid.clone(),
                role: ROLE,
            }))
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

    impl Into<Vec<ResourceInfo>> for Ok {
        fn into(self) -> Vec<ResourceInfo> {
            self.resource
        }
    }

    impl EventMaker for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle<
            St: StateOp,
            Rn: RandomNumber,
            Rt: Runtime,
            Id: RowId,
            Mpsc: MultiProducerSingleConsumer,
            Ed: Coding,
            Rg: Regex,
            Auth: HashedPassword,
            Jwt: JWT,
        >(
            &self,
            side_effects: &mut server_methods::SideEffects,
            state: &mut St,
            jwt: &Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            if Id::validate(&self.user_uuid) {
                if side_effects
                    .authenticated_users
                    .get(&self.user_uuid)
                    .is_none()
                {
                    errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                };
            } else {
                errr.user_uuid = Some(UserUuidError::Invalid);
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let resource = state.read_list_company_and_branch(&self.user_uuid).await?;

            Ok(Ok(Ok { resource }))
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
        pub new_uuid: db_types::UuidType,
        pub branch_name: String,
        pub company_belong: db_types::UuidType,
        pub user_uuid: db_types::UuidType,
        pub currency: db_types::Currency,
        pub location: db_types::Location,
        pub role: db_types::Role,
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

    impl Into<Vec<ResourceInfo>> for Ok {
        fn into(self) -> Vec<ResourceInfo> {
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
                resource: Resource::TableAccessControlForCompanyBranchFieldRole(self.role),
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

            resource
        }
    }

    impl EventMaker for Input {
        type Ok = Ok;
        type Error = Error;

        async fn handle<
            St: StateOp,
            Rn: RandomNumber,
            Rt: Runtime,
            Id: RowId,
            Mpsc: MultiProducerSingleConsumer,
            Ed: Coding,
            Rg: Regex,
            Auth: HashedPassword,
            Jwt: JWT,
        >(
            &self,
            side_effects: &mut server_methods::SideEffects,
            state: &mut St,
            jwt: &Jwt,
        ) -> StdResult<StdResult<Self::Ok, Self::Error>, DynamicError> {
            let mut errr = Self::Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(RowIdError::Invalid);
            }

            if Id::validate(&self.user_uuid) {
                if side_effects
                    .authenticated_users
                    .get(&self.user_uuid)
                    .is_none()
                {
                    errr.user_uuid = Some(UserUuidError::NotAuthenticated);
                };
            } else {
                errr.user_uuid = Some(UserUuidError::Invalid);
            };

            if !Id::validate(&self.company_belong) {
                errr.company_belong = Some(CompanyBelongError::IdInWrongFormat);
            }

            if errr != Self::Error::default() {
                return Ok(Err(errr));
            }

            let (user_roles, is_new_uuid_used, is_company_exist, is_branch_name_used) = state
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

            Ok(Ok(Ok {
                new_uuid: self.new_uuid.clone(),
                branch_name: self.branch_name.clone(),
                company_belong: self.company_belong.clone(),
                user_uuid: self.user_uuid.clone(),
                currency: self.currency.clone(),
                location: self.location.clone(),
                role: ROLE,
            }))
        }
    }
}
