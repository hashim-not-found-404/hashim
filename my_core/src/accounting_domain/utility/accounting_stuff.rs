use serde::Deserialize;
use serde::Serialize;

#[derive(PartialEq, Debug)]
enum CostFlowType {
    InFlow(InFlowType),
    OutFlow(OutFlowType),
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone, Default)]
pub enum OutFlowType {
    None, // reorderable
    QuantityEqualAmount,
    QuantityEqualZero,
    #[default]
    Wac, // reorderable
    Fifo, // sortable
    Lifo, // sortable
    Hifo, // sortable
    Lofo, // sortable
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone, Default)]
pub enum InFlowType {
    #[default]
    None,
    QuantityEqualAmount,
    QuantityEqualZero,
    Wac,
}
