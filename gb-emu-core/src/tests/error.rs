use crate::cartridge::CartridgeError;
use std::convert::From;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum TestError {
    CartridgeError(CartridgeError),
}

impl From<CartridgeError> for TestError {
    fn from(value: CartridgeError) -> Self {
        Self::CartridgeError(value)
    }
}

impl Error for TestError {}

impl Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TestError::CartridgeError(err) => err.to_string(),
        };
        write!(f, "TestError: {}", msg)
    }
}
