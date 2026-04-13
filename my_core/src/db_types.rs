// #[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DataGroup<RowId> {
    Company(RowId),
    Branch(RowId),
}
