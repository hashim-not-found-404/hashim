use crate::prelude::*;

pub struct Atooooooooooo;
impl Coding for Atooooooooooo {
    fn encode<T: Serialize>(data: T) -> Vec<u8> {
        to_allocvec(&data).unwrap().to_vec()
    }

    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, DynamicError> {
        match from_bytes::<T>(data) {
            Ok(text) => return Ok(text),
            Err(err) => return Err(Box::new(err)),
        }
    }
}
