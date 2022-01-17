//! Errors from [Resolver](crate::sql_expr::resolver::Resolver)
use thiserror::Error;

/// Represents all errors from the Resolver
#[derive(Error, Debug)]
pub enum ResolverError {
    /// Aux param can't be resolved, because it's missing.
    #[error("aux param `{0}` is missing")]
    AuxParamMissing(String),
    /// Argument param can't be resolved, because it's missing.
    #[error("not enough arguments provided")]
    ArgumentMissing,
    /// Value of self alias is unknown.
    #[error("unresolved self alias `..`")]
    UnresolvedSelfAlias,
    /// Value of other alias is unknown.
    #[error("unresolved other alias `...`")]
    UnresolvedOtherAlias,
    /// Value of argument is unknown.
    #[error("unresolved argument")]
    UnresolvedArgument,
    /// Value of aux param is unknown.
    #[error("unresolved aux param `{0}`")]
    UnresolvedAuxParameter(String),
}

// Result type alias with [ResolverError]
pub type Result<T> = std::result::Result<T, ResolverError>;