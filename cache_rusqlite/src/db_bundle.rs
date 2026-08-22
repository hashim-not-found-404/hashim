use crate::read_cases;
use crate::read_cases::create_account_for_branch;
use crate::read_cases::create_journal_entry;
use crate::read_cases::get_all_accounts_for_branch;
use crate::utility::cache_adapter;
use my_core::client::cache_op;

pub struct S;

impl cache_op::DbBundle<cache_adapter::S> for S {
    type CreateAccount = read_cases::create_account::S;
    type CreateAccountForBranch = create_account_for_branch::S;
    type CreateCompany = read_cases::create_company::S;
    type CreateCompanyBranch = read_cases::create_company_branch::S;
    type CreateJournalEntry = create_journal_entry::S;
    type GetAllAccountsForBranch = get_all_accounts_for_branch::S;
    type ListCompanyAndBranch = read_cases::list_company_and_branch::S;
    type SignIn = read_cases::sign_in::S;
    type SignUp = read_cases::sign_up::S;
}
