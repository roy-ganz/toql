//! Toql query field information for keys.



use crate::sql_arg::SqlArg;

mod join;

/// The [KeyFields] trait is similar to [Key](crate::key::Key) but provides field names instead of columns.
/// It is used to build key predicates in Toql queries.
pub trait KeyFields {
    type Entity;

    /// Return primary key fields for a given entity.
    fn fields() -> Vec<String>;

    /// Return key values as params. Useful to loop across a composite key.
    fn params(&self) -> Vec<SqlArg>;
}
