
/// A trait to convert a simple datatype into a predicate argument. Used by builder functions. Not very interesting ;)
pub trait PredicateArg {
    fn to_args(self) -> Vec<SqlArg>;
}
