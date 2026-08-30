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
use my_core::server::server_methods::DbBundle;

pub struct S;

impl DbBundle<db_client::S> for S {
    type ReadCreateAccount = create_account::S;
    type ReadCreateAccountForBranch = create_account_for_branch::S;
    type ReadCreateCompany = create_company::S;
    type ReadCreateCompanyBranch = create_company_branch::S;
    type ReadCreateJournalEntry = create_journal_entry::S;
    type ReadGetAllAccounts = get_all_accounts::S;
    type ReadGetAllAccountsForBranch = get_all_accounts_for_branch::S;
    type ReadGetCompaniesAndBranches = get_companies_and_branches::S;
    type ReadSignIn = sign_in::S;
    type ReadSignUp = sign_up::S;
    type WriteCreateAccount = create_account::S;
    type WriteCreateAccountForBranch = create_account_for_branch::S;
    type WriteCreateCompany = create_company::S;
    type WriteCreateCompanyBranch = create_company_branch::S;
    type WriteCreateJournalEntry = create_journal_entry::S;
    type WriteSignUp = sign_up::S;
}
