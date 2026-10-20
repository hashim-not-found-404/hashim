use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}
