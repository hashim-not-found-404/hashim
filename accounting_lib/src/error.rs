#[derive(Debug, PartialEq)]
pub struct Error<MovedInVariables, ErrorType> {
    pub moved_in_variables: MovedInVariables, // this important for type state desgin
    pub error_code: ErrorType,
    pub file: &'static str,
    pub line: u32,
}

#[macro_export]
macro_rules! bail {
    ($moved_in_variables:expr,$error_code:expr) => {
        return Err($crate::error::Error {
            moved_in_variables: $moved_in_variables,
            error_code: $error_code,
            file: file!(),
            line: line!(),
        })
    };
}
