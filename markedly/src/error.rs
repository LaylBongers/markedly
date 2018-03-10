use std::error::{Error as RError};

#[derive(Debug)]
pub enum Error {
    Attribute { component: String, line: usize, field: String, inner: Box<Error> },
    Value { error: String, inner: Option<Box<Error>> },
    Script { error: String },
    Generic { error: Box<RError> },
    Other { error: String },
}

impl Error {
    pub fn new_value(error: &str, inner: Error) -> Self {
        Error::Value {
            error: error.into(),
            inner: Some(Box::new(inner))
        }
    }
}

impl From<::rlua::Error> for Error {
    fn from(error: ::rlua::Error) -> Self {
        Error::Script {
            error: format!("{}", error),
        }
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Other {
            error,
        }
    }
}

impl<'a> From<&'a str> for Error {
    fn from(error: &'a str) -> Self {
        Error::Other {
            error: error.into(),
        }
    }
}
