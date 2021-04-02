use std::error::Error;
use std::fmt;
use std::io;
use sxd_document::parser;

#[derive(Debug)]
pub struct TizenError {
    pub message: String,
}

impl fmt::Display for TizenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TizenError: {}", self.message)
    }
}

impl Error for TizenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<io::Error> for TizenError {
    fn from(error: io::Error) -> Self {
        TizenError {
            message: format!("{}", error),
        }
    }
}

impl From<parser::Error> for TizenError {
    fn from(error: parser::Error) -> Self {
        TizenError {
            message: format!("{}", error),
        }
    }
}

impl From<String> for TizenError {
    fn from(str_value: String) -> Self {
        TizenError { message: str_value }
    }
}
