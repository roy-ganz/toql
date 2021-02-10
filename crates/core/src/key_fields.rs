//! # Key trait
//!
//! The key trait is implemented for every Toql derived struct.
//! The most useful functions for library consumers are [get_key] and [set_key] to access the primary key of a struct.
//! Notice that these operations fail, if the fields that should hold the values are `None`.
//!

use crate::{to_query::ToQuery, sql_arg::SqlArg, query::{field::Field, Query}};

mod join;
mod to_query;

/// Trait to provide the entity type for a key. This is only used
/// for ergonomics of the api.
pub trait KeyFields {
    type Entity;

    /// Return primary key columns for a given entity.
    fn fields() -> Vec<String>;

    /// Return key values as params. Useful to loop across a composite key.
    fn params(&self) -> Vec<SqlArg>;
}




