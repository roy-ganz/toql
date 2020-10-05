use std::fmt;

#[derive(Debug)]
pub enum ResolverError {
    AuxParamMissing(String),
    ArgumentMissing,
    UnresolvedSelfAlias,
    UnresolvedOtherAlias,
    UnresolvedArgument,
    UnresolvedAuxParameter(String)
}


impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResolverError::AuxParamMissing(ref s) => write!(f, "aux param `{}` is missing", s),
            ResolverError::ArgumentMissing => write!(f, "not enough arguments provided"),
            ResolverError::UnresolvedSelfAlias => write!(f, "unresolved self alias `..`"),
            ResolverError::UnresolvedOtherAlias => write!(f, "unresolved other alias `...`"),
            ResolverError::UnresolvedArgument => write!(f, "unresolved argument"),
            ResolverError::UnresolvedAuxParameter(ref s) => write!(f, "unresolved aux param `{}`", s),
        }
    }
}