pub mod target {
    use my_core::accounting_domain::utility::types;
    use my_core::accounting_domain::utility::types::RowId;
    use uuid::Uuid;

    pub struct S;

    impl RowId for S {
        fn parse(s: impl AsRef<str>) -> Option<types::UuidType> {
            let uuid = Uuid::parse_str(s.as_ref()).ok()?;
            Some(types::UuidType(*uuid.as_bytes()))
        }

        fn generate() -> types::UuidType {
            types::UuidType(*Uuid::now_v7().as_bytes())
        }

        fn get_time_as_seconds(uuid: &types::UuidType) -> Option<u64> {
            let u = Uuid::from_bytes(uuid.0);
            u.get_timestamp().map(|ts| ts.to_unix().0)
        }

        fn validate(uuid: &types::UuidType) -> bool {
            let u = Uuid::from_bytes(uuid.0);
            matches!(u.get_version_num(), 7)
        }
    }
}
