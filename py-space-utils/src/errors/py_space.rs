use std::error::Error;

#[derive(Debug)]
pub enum PySpaceError {
    CantGetCurrentPath(String),
    FailedToReadFile(String),
    FailedToParseFile(String),
}

impl std::fmt::Display for PySpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PySpaceError::CantGetCurrentPath(message) => {
                write!(f, "Can't get current path: {}", message)
            }

            PySpaceError::FailedToReadFile(message) => {
                write!(f, "Faild to read file: {}", message)
            }

            PySpaceError::FailedToParseFile(message) => {
                write!(f, "Faild to parse file: {}", message)
            }
        }
    }
}

impl Error for PySpaceError {}
