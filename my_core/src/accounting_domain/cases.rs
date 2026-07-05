use crate::accounting_domain::types;
use serde::{Deserialize, Serialize};

pub trait RowId {
    fn generate() -> types::UuidType;
    fn get_time_as_seconds(uuid: &types::UuidType) -> Option<u64>;
    fn validate(uuid: &types::UuidType) -> bool;
}

pub trait HashedPassword {
    fn sign_up(password: &String) -> String;
    fn sign_in(password: &String, password_hash: &String) -> bool;
}

pub trait JWT {
    fn new() -> Self;
    fn sign(&self, user_uuid: &types::UuidType) -> types::JsonWebTokenType;
    fn validate(&self, token: types::JsonWebTokenType) -> Option<types::UuidType>;
}

pub mod sign_up {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub new_uuid: types::UuidType,
        pub name: Option<String>,
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub new_uuid: types::UuidType,
        pub user_id: String,
        pub user_name: Option<String>,
        pub hashed_password: String,
        jwt: types::JsonWebTokenType,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub new_uuid: Option<types::RowIdError>,
        pub user_id: Option<UserIdError>,
        pub name: Option<String>,
    }

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum UserIdError {
        Duplicated,
    }

    impl Error {
        fn is_empty(&self) -> bool {
            *self != Error::default()
        }
    }

    impl Input {
        fn state_less_check<Id: RowId>(&self) -> Error {
            let mut errr = Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(types::RowIdError::Invalid);
            }

            errr
        }

        fn state_full_check<Id: RowId>(
            &self,
            is_new_uuid_exist: bool,
            is_user_id_exist: bool,
        ) -> Error {
            let mut errr = Error::default();

            if is_new_uuid_exist {
                errr.new_uuid = Some(types::RowIdError::Duplicated);
            }

            if is_user_id_exist {
                errr.user_id = Some(UserIdError::Duplicated);
            }

            errr
        }

        fn handle<Auth: HashedPassword, Jwt: JWT>(&self, jwt: &Jwt) -> Ok {
            let hashed_password = Auth::sign_up(&self.password);
            let jwt = jwt.sign(&self.new_uuid);

            return Ok {
                new_uuid: self.new_uuid.clone(),
                user_id: self.user_id.clone(),
                user_name: self.name.clone(),
                hashed_password,
                jwt,
            };
        }
    }
}

pub mod sign_in {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub user_uuid: types::UuidType,
        pub jwt: types::JsonWebTokenType,
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

    impl Input {
        fn handle<Auth: HashedPassword, Jwt: JWT>(
            &self,
            jwt: &Jwt,
            user_rowid_and_password_hash: &Option<(types::UuidType, String)>,
        ) -> MyResult {
            let mut errr = Error::default();

            let (user_rowid, password_hash) = match user_rowid_and_password_hash {
                Some((user_rowid, password_hash)) => (user_rowid, password_hash),
                None => {
                    errr.user_id = Some(UserIdError::NotExist);
                    return Err(errr);
                }
            };

            match Auth::sign_in(&self.password, password_hash) {
                true => {
                    return Ok(Ok {
                        user_uuid: user_rowid.clone(),
                        jwt: jwt.sign(&user_rowid),
                    });
                }
                false => {
                    errr.password = Some(PasswordError::WrongPassword);
                    return Err(errr);
                }
            };
        }
    }
}

pub mod create_company {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: types::UuidType,
        pub new_uuid: types::UuidType,
        pub company_name: String,
        pub currency: types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub new_uuid: types::UuidType,
        pub company_name: String,
        pub currency: types::Currency,
        pub user_uuid: types::UuidType,
        pub role: types::Role,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<types::UserUuidError>,
        pub new_uuid: Option<types::RowIdError>,
    }

    impl Input {
        fn state_less_check<Id: RowId>(&self) -> Error {
            let mut errr = Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(types::RowIdError::Invalid);
            }

            if !Id::validate(&self.user_uuid) {
                errr.user_uuid = Some(types::UserUuidError::Invalid);
            }
            errr
        }

        fn state_full_check<Id: RowId>(&self, is_new_uuid_used: bool) -> Error {
            let mut errr = Error::default();
            if is_new_uuid_used {
                errr.new_uuid = Some(types::RowIdError::Duplicated);
            }
            errr
        }

        fn handle(&self) -> Ok {
            const ROLE: types::Role = types::Role::Manager;

            Ok {
                new_uuid: self.new_uuid.clone(),
                company_name: self.company_name.clone(),
                currency: self.currency.clone(),
                user_uuid: self.user_uuid.clone(),
                role: ROLE,
            }
        }
    }
}

pub mod list_company_and_branch {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: types::UuidType,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub resource: Vec<types::ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<types::UserUuidError>,
    }

    impl Input {
        fn state_less_check<Id: RowId>(&self) -> Error {
            let mut errr = Error::default();

            if !Id::validate(&self.user_uuid) {
                errr.user_uuid = Some(types::UserUuidError::Invalid);
            }

            errr
        }
    }
}

pub mod create_company_branch {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: types::UuidType,
        pub new_uuid: types::UuidType,
        pub company_belong: types::UuidType,
        pub branch_name: String,
        pub location: types::Location,
        pub currency: types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub new_uuid: types::UuidType,
        pub branch_name: String,
        pub company_belong: types::UuidType,
        pub user_uuid: types::UuidType,
        pub currency: types::Currency,
        pub location: types::Location,
        pub role: types::Role,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub user_uuid: Option<types::UserUuidError>,
        pub new_uuid: Option<types::RowIdError>,
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

    impl Input {
        fn state_less_check<Id: RowId>(&self) -> Error {
            let mut errr = Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(types::RowIdError::Invalid);
            }

            if !Id::validate(&self.user_uuid) {
                errr.user_uuid = Some(types::UserUuidError::Invalid);
            };

            if !Id::validate(&self.company_belong) {
                errr.company_belong = Some(CompanyBelongError::IdInWrongFormat);
            }

            errr
        }

        fn state_full_check<Id: RowId>(
            &self,
            user_roles: &Vec<types::Role>,
            is_new_uuid_used: bool,
            is_company_exist: bool,
            is_branch_name_used: bool,
        ) -> Error {
            let mut errr = Error::default();

            if !types::Role::has_any(
                &user_roles,
                &[types::Role::Manager, types::Role::CoManager],
            ) {
                errr.user_uuid = Some(types::UserUuidError::YouDontHavePermissionToDoThat);
            }

            if is_new_uuid_used {
                errr.new_uuid = Some(types::RowIdError::Duplicated);
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

            errr
        }

        fn handel(&self) -> Ok {
            const ROLE: types::Role = types::Role::CoManager;

            Ok {
                new_uuid: self.new_uuid.clone(),
                branch_name: self.branch_name.clone(),
                company_belong: self.company_belong.clone(),
                user_uuid: self.user_uuid.clone(),
                currency: self.currency.clone(),
                location: self.location.clone(),
                role: ROLE,
            }
        }
    }
}
