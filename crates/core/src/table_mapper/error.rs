use thiserror::Error;

#[derive(Error, Debug)]
/// Represents all errors from the SQL Builder
pub enum TableMapperError {
    /// The requested canonical alias is not used. Contains the alias name.
    #[error("canonical sql alias `{0}` is missing")]
    CanonicalAliasMissing(String),

    /// The column is missing. Contains the table and collumn name.
    #[error("database table `{0}` is missing column `{1}`")]
    ColumnMissing(String, String),
}
