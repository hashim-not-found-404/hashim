use crate::prelude::*;

pub struct Atooooooooooo;
impl Coding for Atooooooooooo {
    type Error = MyError;

    fn encode<T: Serialize>(data: T) -> Vec<u8> {
        to_allocvec(&data).unwrap().to_vec()
    }

    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, Self::Error> {
        match from_bytes::<T>(data) {
            Ok(text) => return Ok(text),
            Err(_) => return Err(MyError::DecodingError),
        }
    }
}
