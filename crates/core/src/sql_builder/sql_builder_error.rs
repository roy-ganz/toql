use std::fmt;

#[derive(Debug)]
/// Represents all errors from the SQL Builder
pub enum SqlBuilderError {
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    FieldMissing(String),
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    PredicateMissing(String),
    /// The join is not mapped to a column or SQL expression. Contains the field name.
    JoinMissing(String),
    /// The selection is not known to the mapper. Contains the field name.
    SelectionMissing(String),
    /// The field requires a role that the query does not have. Contains the role.
    RoleRequired(String),
    /// The filter expects other arguments. Typically raised by custom functions (FN) if the number of arguments is wrong.
    FilterInvalid(String),
    /// A query expression requires a query parameter, that is not provided. Contains the parameter.
    QueryParamMissing(String),
    /// The query parameter that is required by the query expression is wrong. Contains the parameter and the details.
    QueryParamInvalid(String, String),
    /// A predicate requires more arguments, than the toql q uery provided, contains the predicate.
    PredicateArgumentMissing(String),

    /// An key cannot be set, because type is wrong or key is composite key
    KeyMismatch(String, String),
}

impl fmt::Display for SqlBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlBuilderError::FieldMissing(ref s) => write!(f, "field `{}` is missing", s),
            SqlBuilderError::SelectionMissing(ref s) => write!(f, "selection `{}` is missing", s),
            SqlBuilderError::PredicateMissing(ref s) => write!(f, "predicate `@{}` is missing", s),
            SqlBuilderError::JoinMissing(ref s) => write!(f, "join `{}` is missing", s),
            SqlBuilderError::RoleRequired(ref s) => write!(f, "role `{}` is required", s),
            SqlBuilderError::FilterInvalid(ref s) => write!(f, "filter `{}` is invalid ", s),
            SqlBuilderError::KeyMismatch(ref t, ref s) => write!(f, "Key with value `{}` does not match key of `{}` ", t, s),
            SqlBuilderError::QueryParamMissing(ref s) => {
                write!(f, "query parameter `{}` is missing ", s)
            }
            SqlBuilderError::QueryParamInvalid(ref s, ref d) => {
                write!(f, "query parameter `{}` is invalid: {} ", s, d)
            }
            SqlBuilderError::PredicateArgumentMissing(ref s) => {
                write!(f, "predicate `{}` requires more arguments. ", s)
            }
        }
    }
}
