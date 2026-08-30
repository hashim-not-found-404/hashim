use crate::read_write_cases::create_account;
use crate::read_write_cases::create_account_for_branch;
use crate::read_write_cases::create_company;
use crate::read_write_cases::create_company_branch;
use crate::read_write_cases::create_journal_entry;
use crate::read_write_cases::get_all_accounts;
use crate::read_write_cases::get_all_accounts_for_branch;
use crate::read_write_cases::get_companies_and_branches;
use crate::read_write_cases::sign_in;
use crate::read_write_cases::sign_up;
use crate::utility::db_client;
use my_core::server::server_methods;

pub struct S;

impl server_methods::DbBundle<db_client::S> for S {
    type CreateAccount = create_account::S;
    type CreateAccountForBranch = create_account_for_branch::S;
    type CreateCompany = create_company::S;
    type CreateCompanyBranch = create_company_branch::S;
    type CreateJournalEntry = create_journal_entry::S;
    type GetAllAccounts = get_all_accounts::S;
    type GetAllAccountsForBranch = get_all_accounts_for_branch::S;
    type GetCompaniesAndBranches = get_companies_and_branches::S;
    type SignIn = sign_in::S;
    type SignUp = sign_up::S;
    type WriteCreateAccount = create_account::S;
    type WriteCreateAccountForBranch = create_account_for_branch::S;
    type WriteCreateCompany = create_company::S;
    type WriteCreateCompanyBranch = create_company_branch::S;
    type WriteCreateJournalEntry = create_journal_entry::S;
    type WriteSignUp = sign_up::S;
}
