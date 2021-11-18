use super::error::DeriveError;

pub type Result<T> = std::result::Result<T, DeriveError>;
