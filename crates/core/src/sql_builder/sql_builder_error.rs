//! Errors from [SqlBuilder](crate::sql_builder::SqlBuilder)
use thiserror::Error;

/// Represents all errors from the SQL Builder
#[derive(Error, Debug)]
pub enum SqlBuilderError {
    /// The field is not mapped to a column or SQL expression. Contains the field name.
    #[error("field `{0}` is missing")]
    FieldMissing(String),

    /// The field is not mapped to a column or SQL expression. Contains the field name.
    #[error("predicate `@{0}` is missing")]
    PredicateMissing(String),

    /// The join is not mapped to a column or SQL expression. Contains the join name and the table name.
    #[error("join `{0}` is missing on mapper for table `{1}`")]
    JoinMissing(String, String),

    /// The merge is not mapped. Contains the field name.
    #[error("merge `{0}` is missing")]
    MergeMissing(String),

    /// The selection is not known to the mapper. Contains the field name.
    #[error("selection `{0}` is missing")]
    SelectionMissing(String),

    /// The field requires a role that the query does not have. Contains the role and the query_path.
    #[error("role expression `{0}` failed for `{1}`")]
    RoleRequired(String, String),

    /// The filter expects other arguments. Typically raised by custom functions (FN) if the number of arguments is wrong.
    #[error("filter `{0}` is invalid")]
    FilterInvalid(String),

    /// A query expression requires a query parameter, that is not provided. Contains the parameter.
    #[error("query parameter `{0}` is missing")]
    QueryParamMissing(String),

    /// The query parameter that is required by the query expression is wrong. Contains the parameter and the details.
    #[error("query parameter `{0}` is invalid: {1}")]
    QueryParamInvalid(String, String),

    /// A predicate requires more arguments, than the toql q uery provided, contains the predicate.
    #[error("predicate `{0}` requires more arguments")]
    PredicateArgumentMissing(String),

    /// An key cannot be set, because type is wrong or key is composite key
    #[error("key with value `{0}` does not match key of `{1}`")]
    KeyMismatch(String, String),

    /// A path was found for a selection that only exists in root, such as $all, $mut, $cnt
    #[error("a path `{0}` was found but no path is allowed")]
    PathUnexpected(String),
}