use crate::read_write_cases;
use crate::read_write_cases::create_account_for_branch;
use crate::read_write_cases::create_journal_entry;
use crate::read_write_cases::get_all_accounts_for_branch;
use crate::utility::cache_adapter;
use my_core::client::cache_op;

pub struct S;

impl cache_op::DbBundle<cache_adapter::S> for S {
    type CreateAccount = read_write_cases::create_account::S;
    type CreateAccountForBranch = create_account_for_branch::S;
    type CreateCompany = read_write_cases::create_company::S;
    type CreateCompanyBranch = read_write_cases::create_company_branch::S;
    type CreateJournalEntry = create_journal_entry::S;
    type GetAllAccounts = read_write_cases::get_all_accounts::S;
    type GetAllAccountsForBranch = get_all_accounts_for_branch::S;
    type ListCompanyAndBranch = read_write_cases::list_company_and_branch::S;
    type SignIn = read_write_cases::sign_in::S;
    type SignUp = read_write_cases::sign_up::S;
    type WriteCreateAccount = read_write_cases::create_account::S;
    type WriteCreateAccountForBranch = read_write_cases::create_account_for_branch::S;
    type WriteCreateCompany = read_write_cases::create_company::S;
    type WriteCreateCompanyBranch = read_write_cases::create_company_branch::S;
    type WriteCreateJournalEntry = read_write_cases::create_journal_entry::S;
    type WriteGetAllAccounts = read_write_cases::get_all_accounts::S;
    type WriteGetAllAccountsForBranch = read_write_cases::get_all_accounts_for_branch::S;
    type WriteListCompanyAndBranch = read_write_cases::list_company_and_branch::S;
    type WriteSignIn = read_write_cases::sign_in::S;
    type WriteSignUp = read_write_cases::sign_up::S;
}
