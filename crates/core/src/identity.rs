//! # Identity trait
//!
//! The identity trait is used to update foreign keys.
//! The trait is used by the TreeIdentity trait and implemented for all Toql derived entities.
//! Key columns in Toql entities are automatically identity columns.
//! Other columns may be marked in addition to ensure proper key refreshing when doing inserts.

use crate::sql_arg::SqlArg;

/// Trait to provide the entity type for a key. This is only used
/// for ergonomics of the api.
pub trait Identity {
    /// Returns primary key columns for a given entity.
    fn columns() -> Vec<String>;

    /// Sets the value for an identity column
    fn set_column(&mut self, column: &str, value: &SqlArg) -> crate::result::Result<()>;
}
