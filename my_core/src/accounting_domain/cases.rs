use crate::accounting_domain::types;
use serde::{Deserialize, Serialize};

pub trait RowId: 'static {
    fn generate() -> types::UuidType;
    fn get_time_as_seconds(uuid: &types::UuidType) -> Option<u64>;
    fn validate(uuid: &types::UuidType) -> bool;
}

pub trait HashedPassword {
    fn sign_up(password: &String) -> String;
    fn sign_in(password: &String, password_hash: &String) -> bool;
}

pub trait JWT: 'static {
    fn new() -> Self;
    fn sign(&self, user_uuid: &types::UuidType) -> types::JsonWebTokenType;
    fn validate(&self, token: types::JsonWebTokenType) -> Option<types::UuidType>;
}

pub(crate) trait MyErrorTrait: Default + PartialEq {
    fn is_there_error(&self) -> bool {
        *self != Self::default()
    }
}

pub mod sign_up {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub(crate) new_uuid: types::UuidType,
        pub(crate) name: Option<String>,
        pub(crate) user_id: String,
        pub(crate) password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub new_uuid: types::UuidType,
        pub user_id: String,
        pub user_name: Option<String>,
        pub hashed_password: String,
        pub(crate) jwt: types::JsonWebTokenType,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub(crate) new_uuid: Option<types::RowIdError>,
        pub(crate) user_id: Option<UserIdError>,
        pub(crate) name: Option<String>,
    }

    impl MyErrorTrait for Error {}

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub(crate) enum UserIdError {
        Duplicated,
    }

    impl Input {
        pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
            let mut errr = Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(types::RowIdError::Invalid);
            }

            errr
        }

        pub(crate) fn state_full_check<Id: RowId>(
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

        pub(crate) fn state_full_operation<Auth: HashedPassword, Jwt: JWT>(&self, jwt: &Jwt) -> Ok {
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

pub(crate) mod sign_in {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub(crate) user_id: String,
        pub(crate) password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub user_uuid: types::UuidType,
        pub user_id: String,
        pub user_name: Option<String>,
        pub(crate) jwt: types::JsonWebTokenType,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub(crate) user_id: Option<UserIdError>,
        pub(crate) password: Option<PasswordError>,
    }

    impl MyErrorTrait for Error {}

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub(crate) enum UserIdError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub(crate) enum PasswordError {
        WrongPassword,
    }

    impl Input {
        pub(crate) fn state_full_check<Auth: HashedPassword, Jwt: JWT>(
            &self,
            jwt: &Jwt,
            user_rowid_and_password_hash_and_name: &Option<(
                types::UuidType,
                String,
                Option<String>,
            )>,
        ) -> MyResult {
            let mut errr = Error::default();

            let (user_rowid, password_hash, user_name) = match user_rowid_and_password_hash_and_name
            {
                Some((user_rowid, password_hash, user_name)) => {
                    (user_rowid, password_hash, user_name)
                }
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
                        user_id: self.user_id.clone(),
                        user_name: user_name.clone(),
                    });
                }
                false => {
                    errr.password = Some(PasswordError::WrongPassword);
                    return Err(errr);
                }
            };
        }

        pub(crate) fn state_full_operation(
            &self,
            jwt: &types::JsonWebTokenType,
            user_uuid: &types::UuidType,
            user_name: &Option<String>,
        ) -> Ok {
            return Ok {
                user_uuid: user_uuid.clone(),
                user_id: self.user_id.clone(),
                user_name: user_name.clone(),
                jwt: jwt.clone(),
            };
        }
    }
}

pub mod create_company {
    use super::*;

    pub type MyResult = Result<Ok, Error>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub(crate) user_uuid: types::UuidType,
        pub(crate) new_uuid: types::UuidType,
        pub(crate) company_name: String,
        pub(crate) currency: types::Currency,
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
        pub(crate) user_uuid: Option<types::UserUuidError>,
        pub(crate) new_uuid: Option<types::RowIdError>,
    }

    impl MyErrorTrait for Error {}

    impl Input {
        pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
            let mut errr = Error::default();

            if !Id::validate(&self.new_uuid) {
                errr.new_uuid = Some(types::RowIdError::Invalid);
            }

            if !Id::validate(&self.user_uuid) {
                errr.user_uuid = Some(types::UserUuidError::Invalid);
            }
            errr
        }

        pub(crate) fn state_full_check<Id: RowId>(&self, is_new_uuid_used: bool) -> Error {
            let mut errr = Error::default();
            if is_new_uuid_used {
                errr.new_uuid = Some(types::RowIdError::Duplicated);
            }
            errr
        }

        pub(crate) fn state_less_operation(&self) -> Ok {
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
        pub(crate) user_uuid: types::UuidType,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub(crate) user_uuid: types::UuidType, // <-- add this
        pub(crate) data: Vec<AllCompaniesThatUserInWithRoles>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct AllCompaniesThatUserInWithRoles {
        pub company_uuid: types::UuidType,
        pub company_name: String,
        pub company_currancy: types::Currency,
        pub user_roles: Vec<types::Role>,
        pub branches: Vec<AllBranchesThatUserInWithRoles>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct AllBranchesThatUserInWithRoles {
        pub branch_uuid: types::UuidType,
        pub branch_name: String,
        pub branch_currancy: types::Currency,
        pub user_roles: Vec<types::Role>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
    pub struct Error {
        pub(crate) user_uuid: Option<types::UserUuidError>,
    }

    impl MyErrorTrait for Error {}

    impl Input {
        pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
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
        pub(crate) user_uuid: types::UuidType,
        pub(crate) new_uuid: types::UuidType,
        pub(crate) company_belong: types::UuidType,
        pub(crate) branch_name: String,
        pub(crate) location: types::Location,
        pub(crate) currency: types::Currency,
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
        pub(crate) user_uuid: Option<types::UserUuidError>,
        pub(crate) new_uuid: Option<types::RowIdError>,
        pub(crate) company_belong: Option<CompanyBelongError>,
        pub(crate) branch_name: Option<BranchNameError>,
        pub(crate) location: Option<LocationError>,
    }

    impl MyErrorTrait for Error {}

    // utility types

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub(crate) enum CompanyBelongError {
        IdInWrongFormat,
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub(crate) enum BranchNameError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub(crate) enum LocationError {
        Invalid,
    }

    impl Input {
        pub(crate) fn state_less_check<Id: RowId>(&self) -> Error {
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

        pub(crate) fn state_full_check<Id: RowId>(
            &self,
            user_roles: &Vec<types::Role>,
            is_new_uuid_used: bool,
            is_company_exist: bool,
            is_branch_name_used: bool,
        ) -> Error {
            let mut errr = Error::default();

            if !types::Role::has_any(&user_roles, &[types::Role::Manager, types::Role::CoManager]) {
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

        pub(crate) fn state_less_operation(&self) -> Ok {
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
