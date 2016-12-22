use std::error::Error;
use std::fmt::{self, Debug};

#[derive(Debug)]
pub struct StringError {
    pub description: String,
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.description, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &*self.description
    }
}

#[derive(Debug)]
pub struct AuthenticationError {}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt("Authentication failed.", f)
    }
}

impl Error for AuthenticationError {
    fn description(&self) -> &str {
        &*"Authentication failed"
    }
}
