use crate::read_write_cases::{
    create_account, create_company, create_company_branch, list_company_and_branch, sign_in,
    sign_up, utils::db_client,
};
use my_core::server::server_methods;

pub struct S;

impl server_methods::DbBundle<db_client::S> for S {
    type CreateAccount = create_account::S;
    type CreateCompany = create_company::S;
    type CreateCompanyBranch = create_company_branch::S;
    type ListCompanyAndBranch = list_company_and_branch::S;
    type SignIn = sign_in::S;
    type SignUp = sign_up::S;
}
