pub mod m {
    use crate::prelude::*;
    pub struct S;

    impl CacheIO for S {
        async fn new() -> Result<Self, DynamicError> {
            todo!()
        }

        async fn write_data(&self, data: &data_receiver::Input) -> Result<(), DynamicError> {
            todo!()
        }
    }
}
