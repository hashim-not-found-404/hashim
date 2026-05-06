pub mod m {
    use crate::prelude::*;
    pub struct S;

    impl CacheIO for S {
        async fn new() -> Result<Self, DynamicError> {
            mbg!("here is the error was");
            Ok(S)
        }

        async fn write_data(&self, data: &data_receiver::Input) -> Result<(), DynamicError> {
            todo!()
        }
    }
}
