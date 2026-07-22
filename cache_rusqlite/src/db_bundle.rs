use my_core::accounting_client::cache_op;

use crate::read_cases::{self};
use crate::utility::cache_adapter;

pub struct S;

impl cache_op::DbBundle<cache_adapter::S> for S {
    type CreateAccount = read_cases::create_account::S;
    type CreateCompany = read_cases::create_company::S;
    type CreateCompanyBranch = read_cases::create_company_branch::S;
    type ListCompanyAndBranch = read_cases::list_company_and_branch::S;
    type SignIn = read_cases::sign_in::S;
    type SignUp = read_cases::sign_up::S;
}
