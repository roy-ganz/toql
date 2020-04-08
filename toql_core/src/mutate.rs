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

use crate::error::ToqlError;
use crate::key::Keyed;
use core::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::result::Result;
use crate::sql_mapper::Mapped;


/// Trait for delete functions (They work with entity keys).
pub trait Delete<'a, T: crate::key::Keyed + 'a> {
    type Error;
    /// Delete one structs, returns tuple with SQL statement and SQL params or error.
    fn delete_one_sql(
        key: T::Key,
        roles: &HashSet<String>,
    ) -> Result<(String, Vec<String>), Self::Error>
    where
        T: crate::key::Keyed + crate::sql_mapper::Mapped +  'a,
    {
        Ok(Self::delete_many_sql(crate::key::sql_predicate(&[key], &<T as  crate::sql_mapper::Mapped>::table_alias()), roles)?.unwrap())
    }
    /// Delete many structs, returns tuple with SQL statement and SQL params or error.
    fn delete_many_sql(
        predicate: (String, Vec<String>),
        roles: &HashSet<String>,
    ) -> Result<Option<(String, Vec<String>)>, Self::Error>;
}

/// Trait for update. They work with entities
pub trait Update<'a, T: 'a> {
    type Error;
    fn update_one_sql<Q: Borrow<T>>(
        entity: Q,
        roles: &HashSet<String>,
    ) -> Result<Option<(String, Vec<String>)>, Self::Error> {
        Self::update_many_sql(&[entity], roles)
    }
    /// Update many structs, returns tuple with SQL statement and SQL params or error.
    fn update_many_sql<Q: Borrow<T>>(
        entities: &[Q],
        roles: &HashSet<String>,
    ) -> Result<Option<(String, Vec<String>)>, Self::Error>;
}

/// Trait for update. They work with entities
pub trait Diff<'a, T: 'a> {
    type Error;
    /// Update difference of two structs, given as tuple (old, new), returns a vectro with SQL statements and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    /// Because merged structs are also considered, the returned SQL statements, can be insert, update and delete statements.
    fn full_diff_one_sql<Q: Borrow<T>>(
        outdated: Q,
        updated: Q,
        roles: &HashSet<String>,
    ) -> Result<Vec<(String, Vec<String>)>, Self::Error> {
        Ok(Self::full_diff_many_sql(&[(outdated, updated)], roles)?.unwrap())
    }

    /// Update difference of two structs, given as tuple (old, new), returns tuples with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    fn full_diff_many_sql<Q: Borrow<T>>(
        entities: &[(Q, Q)],
        roles: &HashSet<String>,
    ) -> Result<Option<Vec<(String, Vec<String>)>>, Self::Error>;

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup`  or set selectable fields to None in updated entity.
    fn diff_one_sql<Q: Borrow<T>>(
        outdated: Q,
        updated: Q,
        roles: &HashSet<String>,
    ) -> Result<Option<(String, Vec<String>)>, Self::Error> {
        Self::diff_many_sql(&[(outdated, updated)], roles)
    }

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    fn diff_many_sql<Q: Borrow<T>>(
        entities: &[(Q, Q)],
        roles: &HashSet<String>,
    ) -> Result<Option<(String, Vec<String>)>, Self::Error>;
}

/// Defines a strategy to resolve the conflict, when the record to insert already exists.
pub enum DuplicateStrategy {
    /// Fail and return an error.
    Fail,
    /// Do not insert the record.
    Skip,
    /// Do not insert, but update the record instead.
    Update,
}

/// Trait for insert. They work with entities.
/// This trait is implemented if keys of an entity are inserted too. This is typically the case for association tables.
/// Conflicts can happed if the keys already exist. A strategy must be provided to tell how to resolve the conflict.
pub trait Insert<'a, T: 'a> {
    type Error;

    /// Insert one struct, returns tuple with SQL statement and SQL params or error.
    fn insert_one_sql<Q: Borrow<T>>(
        entity: Q,
        strategy: DuplicateStrategy,
        roles: &HashSet<String>,
    ) -> Result<(String, Vec<String>), Self::Error> {
        Ok(Self::insert_many_sql(&[entity], strategy, roles)?.unwrap())
    }
    /// Insert many structs, returns tuple with SQL statement and SQL params, none if no entities are provided or error.
    fn insert_many_sql<Q: Borrow<T>>(
        entities: &[Q],
        strategy: DuplicateStrategy,
        roles: &HashSet<String>,
    ) -> Result<Option<(String, Vec<String>)>, Self::Error>;
}

/// Marker Trait for insert. They work with entities.
/// This trait is always implemented, but recommened to use if all keys are skipped for insert (use of autovalue).
/// Insert conflicts cannot happed, if all keys are annotated with #[toql(skip_mut)].
pub trait InsertDuplicate {}

/// Update difference of two collections
/// Compares multiple tuples with outdated / current collections and builds insert / update / delete statements
/// to save the changes in a database.
///
///
/// Returns three tuples for insert / update / delete, each containing the SQL statement and parameters.

pub fn collection_delta_sql<'a, T, I, U, D, E>(
    outdated: &'a [T],
    updated: &'a [T],
    roles: &HashSet<String>,
) -> Result<
    (
        Option<(String, Vec<String>)>,
        Option<(String, Vec<String>)>,
        Option<(String, Vec<String>)>,
    ),
    E,
>
where
    T: 'a + Keyed + Mapped,
    I: Insert<'a, T>,
    U: Diff<'a, T>,
    D: Delete<'a, T>,
    E: std::convert::From<<U as Diff<'a, T>>::Error>
        + std::convert::From<<I as Insert<'a, T>>::Error>
        + std::convert::From<<D as Delete<'a, T>>::Error>
        + std::convert::From<ToqlError>,
{
    let mut insert: Vec<&T> = Vec::new();
    let mut diff: Vec<(&'a T, &'a T)> = Vec::new();
    let mut delete: Vec<T::Key> = Vec::new();
    let (mut ins, mut di, mut de) = collections_delta::<T, E>(&vec![(outdated, updated)])?;
    insert.append(&mut ins);
    diff.append(&mut di);
    delete.append(&mut de);

    let insert_sql = <I as Insert<T>>::insert_many_sql(&insert, DuplicateStrategy::Fail, roles)?;
    let diff_sql = <U as Diff<T>>::diff_many_sql(&diff, roles)?;
    let delete_sql = <D as Delete<T>>::delete_many_sql( crate::key::sql_predicate(&delete, &<T as Mapped>::table_alias()) , roles)?;
    Ok((insert_sql, diff_sql, delete_sql))
}

pub fn collections_delta<'a, T, E>(
    collections: &[(&'a [T], &'a [T])],
) -> Result<(Vec<&'a T>, Vec<(&'a T, &'a T)>, Vec<T::Key>), E>
where
    T: 'a + Keyed,
    E: std::convert::From<ToqlError>,
{
    let mut diff: Vec<(&'a T, &'a T)> = Vec::new(); // Vector with entities to diff
    let mut insert: Vec<&'a T> = Vec::new(); // Vector with entities to insert
    let mut delete: Vec<T::Key> = Vec::new(); // Vector with keys to delete

    for (previous_coll, current_coll) in collections {
        let mut previous_index: HashMap<T::Key, &T> = HashMap::new();
        for previous in *previous_coll {
            // Build up index
            let k = Keyed::try_get_key(previous)?;
            previous_index.insert(k, previous);
        }

        for current in *current_coll {
            if previous_index.contains_key(&Keyed::try_get_key(current)?) {
                let previous = previous_index.remove(&Keyed::try_get_key(current)?).unwrap();
                diff.push((previous, current));
            } else {
                insert.push(current);
            }
        }

        for (_k, v) in previous_index {
            delete.push(Keyed::try_get_key(v)?);
        }
    }

    Ok((insert, diff, delete))
}
