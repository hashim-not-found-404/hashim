use infrastructure::random_number::RandomNumber;
use infrastructure::random_number::Rn;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProcessId(u16);

impl Default for ProcessId {
    fn default() -> Self {
        ProcessId(Rn::generate() as u16)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UserConsent {
    WaitForServerResponse,
    DontWaitForServerResponse,
    CancelOperation,
}
