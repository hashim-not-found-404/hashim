use crate::accounting_domain::db_types;
use std::collections::{HashMap, HashSet};

pub struct AllRoles {
    pub companies: HashMap<
        db_types::UuidType, // company uuid
        HashMap<
            db_types::UuidType, // user uuid
            Vec<db_types::Role>,
        >,
    >,
    pub branches: HashMap<
        db_types::UuidType, // branch uuid
        HashMap<
            db_types::UuidType, // user uuid
            Vec<db_types::Role>,
        >,
    >,
}

pub(crate) type ListOfResources = HashMap<db_types::UuidType, Vec<db_types::ResourceInfo>>;

#[derive(Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users: HashSet<db_types::UuidType>,
    pub(crate) users_to_resubscribe: HashSet<db_types::UuidType>,
    pub(crate) resource_to_broadcast_for_company: ListOfResources,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources,
}
