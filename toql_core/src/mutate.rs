//! # Insert / Delete / Update
//!
//! The database support of the Toql derive can build functions to insert, delete and update structs.
//!
//! ## Example (Read the guide for derive attributes)
//!
//! ``` ignore
//! #[derive(Debug, PartialEq, Toql)]
//! struct NewUser {
//!     #[toql(skip_inup)]
//!     id: u8, // Auto value, no insert
//!     username: Option<String>,
//! }
//!
//! let u = NewUser {
//!             id: 5,
//!             username: Some(String::from("Foo")),
//!         };
//!
//! let (sql, params) = NewUser::insert_one_sql(&u)?.unwrap();
//!
//! assert_eq!("INSERT INTO NewUser (username) VALUES (?)", sql);
//! assert_eq!(["Foo"], *params);
//! ```
//!
//! Note that operations are not cascaded. If you insert a struct `Foo` that contains another struct `Bar` only `Foo will be inserted.
//! To deal with those dependencies, you are expected to make multiple calls.
//!
//! If you *update* a struct, fields of type `Option<>` with value `None` are skipped. Read the guide for details!
//!

use crate::error::Result;

/// Trait for insert delete and update functions.
pub trait Mutate<'a, T: 'a> {

    /// Insert one struct, returns tuple with SQL statement and SQL params or error.
    fn insert_one_sql(entity: &'a T) -> Result<Option<(String, Vec<String>)>> {
        Self::insert_many_sql(std::iter::once(entity))
    }
    /// Insert many structs, returns tuple with SQL statement and SQL params or error.
    fn insert_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a;
    /// Delete one structs, returns tuple with SQL statement and SQL params or error.
    fn delete_one_sql(entity: &'a T) -> Result<Option<(String, Vec<String>)>> {
        Self::delete_many_sql(std::iter::once(entity))
    }
    /// Delete many structs, returns tuple with SQL statement and SQL params or error.
    fn delete_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a;
    /// Update one struct, returns tuple with SQL statement and SQL params or error.
    fn update_one_sql(entity: &'a T) -> Result<Option<(String, Vec<String>)>> {
        Self::update_many_sql(std::iter::once(entity))
    }
    /// Update many structs, returns tuple with SQL statement and SQL params or error.
    fn update_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a + Clone;

    /// Update difference of two structs, given as tuple (old, new), returns tuples with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup`
   fn diff_many_sql<I> (entities: I) -> Result<Option<Vec<(String,Vec<String>)>>>
    where I: IntoIterator<Item = (&'a T, &'a T)>  +'a +  Clone;

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup`
    fn shallow_diff_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = (&'a T, &'a T)> + 'a + Clone;
}
