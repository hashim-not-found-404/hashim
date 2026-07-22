use crate::read_write_cases::create_account;
use crate::read_write_cases::create_company;
use crate::read_write_cases::create_company_branch;
use crate::read_write_cases::list_company_and_branch;
use crate::read_write_cases::sign_in;
use crate::read_write_cases::sign_up;
use crate::utility::db_client;
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
