//! Errors from [Resolver](crate::sql_expr::resolver::Resolver)
use std::fmt;

/// Represents all errors from the Resolver
#[derive(Debug)]
pub enum ResolverError {
    /// Aux param can't be resolved, because it's missing.
    AuxParamMissing(String),
    /// Argument param can't be resolved, because it's missing.
    ArgumentMissing,
    /// Value of self alias is unknown.
    UnresolvedSelfAlias,
    /// Value of other alias is unknown.
    UnresolvedOtherAlias,
    /// Value of argument is unknown.
    UnresolvedArgument,
    /// Value of aux param is unknown.
    UnresolvedAuxParameter(String),
}

// Result type alias with [ResolverError]
pub type Result<T> = std::result::Result<T, ResolverError>;

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResolverError::AuxParamMissing(ref s) => write!(f, "aux param `{}` is missing", s),
            ResolverError::ArgumentMissing => write!(f, "not enough arguments provided"),
            ResolverError::UnresolvedSelfAlias => write!(f, "unresolved self alias `..`"),
            ResolverError::UnresolvedOtherAlias => write!(f, "unresolved other alias `...`"),
            ResolverError::UnresolvedArgument => write!(f, "unresolved argument"),
            ResolverError::UnresolvedAuxParameter(ref s) => {
                write!(f, "unresolved aux param `{}`", s)
            }
        }
    }
}
