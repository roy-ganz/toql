
#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum TableMapperError {
    /// The requested canonical alias is not used. Contains the alias name.
    CanonicalAliasMissing(String),
    ColumnMissing(String, String), // table column
}
impl fmt::Display for TableMapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TableMapperError::CanonicalAliasMissing(ref s) => {
                write!(f, "canonical sql alias `{}` is missing", s)
            }
            TableMapperError::ColumnMissing(ref t, ref c) => {
                write!(f, "database table `{}` is missing column `{}`", t, c)
            }
        }
    }
}
