//! Errors from [SqlBuilder](crate::sql_builder::SqlBuilder)
use std::fmt;

/// Represents all errors from the SQL Builder
#[derive(Debug, PartialEq)]
pub enum SqlBuilderError {
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    FieldMissing(String),
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    PredicateMissing(String),
    /// The join is not mapped to a column or SQL expression. Contains the join name and the table name.
    JoinMissing(String, String),
    /// The merge is not mapped. Contains the field name.
    MergeMissing(String),
    /// The selection is not known to the mapper. Contains the field name.
    SelectionMissing(String),
    /// The field requires a role that the query does not have. Contains the role and the query_path.
    RoleRequired(String, String),
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

    /// A path was found for a selection that only exists in root, such as $all, $mut, $cnt
    PathUnexpected(String),
}

impl fmt::Display for SqlBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SqlBuilderError::FieldMissing(ref field) => write!(f, "field `{}` is missing", field),
            SqlBuilderError::PathUnexpected(ref path) => {
                write!(f, "a path `{}` was found but no path is allowed", path)
            }
            SqlBuilderError::SelectionMissing(ref selection) => {
                write!(f, "selection `{}` is missing", selection)
            }
            SqlBuilderError::PredicateMissing(ref predicate) => {
                write!(f, "predicate `@{}` is missing", predicate)
            }
            SqlBuilderError::JoinMissing(ref join, ref table) => {
                write!(
                    f,
                    "join `{}` is missing on mapper for table `{}`",
                    join, table
                )
            }
            SqlBuilderError::MergeMissing(ref merge) => write!(f, "merge `{}` is missing", merge),
            SqlBuilderError::RoleRequired(ref role, ref what) => {
                write!(f, "role expression `{}` failed for {}", role, what)
            }
            SqlBuilderError::FilterInvalid(ref filter) => {
                write!(f, "filter `{}` is invalid ", filter)
            }
            SqlBuilderError::KeyMismatch(ref value, ref ty) => {
                write!(
                    f,
                    "key with value `{}` does not match key of `{}` ",
                    value, ty
                )
            }
            SqlBuilderError::QueryParamMissing(ref param) => {
                write!(f, "query parameter `{}` is missing ", param)
            }
            SqlBuilderError::QueryParamInvalid(ref param, ref reason) => {
                write!(f, "query parameter `{}` is invalid: {} ", param, reason)
            }
            SqlBuilderError::PredicateArgumentMissing(ref predicate) => {
                write!(f, "predicate `{}` requires more arguments. ", predicate)
            }
        }
    }
}
