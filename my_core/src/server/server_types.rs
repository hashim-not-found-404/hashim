use crate::accounting_domain::types;
use std::collections::{HashMap, HashSet};

pub struct AllRoles {
    pub companies: HashMap<
        types::UuidType, // company uuid
        HashMap<
            types::UuidType, // user uuid
            Vec<types::Role>,
        >,
    >,
    pub branches: HashMap<
        types::UuidType, // branch uuid
        HashMap<
            types::UuidType, // user uuid
            Vec<types::Role>,
        >,
    >,
}

pub(crate) type ListOfResources = HashMap<types::UuidType, Vec<types::ResourceInfo>>;

#[derive(Default)]
pub(crate) struct SideEffects {
    pub(crate) authenticated_users: HashSet<types::UuidType>,
    pub(crate) users_to_resubscribe: HashSet<types::UuidType>,
    pub(crate) resource_to_broadcast_for_company: ListOfResources,
    pub(crate) resource_to_broadcast_for_branch: ListOfResources,
}
