use std::str::FromStr;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("`{0}`")]
    Database(String),
    #[error("`{0}`")]
    Other(String),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error("unknown core error")]
    Unknown,
    #[error("unreachable logic")]
    Unreachable,
}

impl FromStr for CoreError {
    type Err = Self;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Other(s.to_owned()))
    }
}
