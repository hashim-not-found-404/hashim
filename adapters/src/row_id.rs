pub mod target {
    use my_core::accounting_domain::utility::types::RowId;
    use my_core::accounting_domain::utility::uuid::UuidType;
    use uuid::Uuid;

    pub struct S;

    impl RowId for S {
        fn parse(s: impl AsRef<str>) -> Option<UuidType> {
            let uuid = Uuid::parse_str(s.as_ref()).ok()?;
            Some(UuidType(*uuid.as_bytes()))
        }

        fn generate() -> UuidType {
            UuidType(*Uuid::now_v7().as_bytes())
        }

        fn get_time_as_seconds(uuid: &UuidType) -> Option<u64> {
            let u = Uuid::from_bytes(uuid.0);
            u.get_timestamp().map(|ts| ts.to_unix().0)
        }

        fn validate(uuid: &UuidType) -> bool {
            let u = Uuid::from_bytes(uuid.0);
            matches!(u.get_version_num(), 7)
        }
    }
}
