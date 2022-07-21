use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MCP2210Error(pub String);

impl fmt::Display for MCP2210Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There was an error during communication with MCP2210: {}", self.0)
    }
}

impl Error for MCP2210Error {}

impl From<libloading::Error> for MCP2210Error {
    fn from(error: libloading::Error) -> Self {
        MCP2210Error(error.to_string())
    }
}