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
use std::collections::HashMap;
use crate::key::Key;




/// Trait for delete functions (They work with entity keys).
pub trait Delete<'a, T: crate::key::Key + 'a> {
     /// Delete one structs, returns tuple with SQL statement and SQL params or error.
    fn delete_one_sql(key: T::Key) -> Result<(String, Vec<String>)> 
    where T: crate::key::Key + 'a
    {
        Ok(Self::delete_many_sql(std::iter::once(key))?.unwrap())
    }
    /// Delete many structs, returns tuple with SQL statement and SQL params or error.
    fn delete_many_sql<I>(keys: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = T::Key> + 'a;
}

/// Trait for update. They work with entities
pub trait Update<'a, T: 'a> {
    /* /// Insert one struct, returns tuple with SQL statement and SQL params or error.
    fn insert_one_sql(entity: &'a T) -> Result<(String, Vec<String>)> {
        Ok(Self::insert_many_sql(std::iter::once(entity))?.unwrap())
    }
    /// Insert many structs, returns tuple with SQL statement and SQL params, none if no entities are provided or error.
    fn insert_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a; */

  /*   /// Delete one structs, returns tuple with SQL statement and SQL params or error.
    fn delete_one_sql(entity: &'a T) -> Result<(String, Vec<String>)> {
        Ok(Self::delete_many_sql(std::iter::once(entity))?.unwrap())
    }
    /// Delete many structs, returns tuple with SQL statement and SQL params or error.
    fn delete_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a; */
    /// Update one struct, returns tuple with SQL statement and SQL params or error.
    /// Returns None, if no updates are required.
    
    fn update_one_sql(entity: &'a T) -> Result<Option<(String, Vec<String>)>> {
        Self::update_many_sql(std::iter::once(entity))
    }
    /// Update many structs, returns tuple with SQL statement and SQL params or error.
    fn update_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a + Clone;

    /// Update difference of two structs, given as tuple (old, new), returns a vectro with SQL statements and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    /// Because merged structs are also considered, the returned SQL statements, can be insert, update and delete statements.
    fn diff_one_sql(outdated: &'a T, updated: &'a T) -> Result<Vec<(String, Vec<String>)>> {
        Ok(Self::diff_many_sql(std::iter::once((outdated, updated)))?.unwrap())
    }

    /// Update difference of two structs, given as tuple (old, new), returns tuples with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    fn diff_many_sql<I>(entities: I) -> Result<Option<Vec<(String, Vec<String>)>>>
    where
        I: IntoIterator<Item = (&'a T, &'a T)> + 'a + Clone;

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup`  or set selectable fields to None in updated entity.
    fn shallow_diff_one_sql(
        outdated: &'a T,
        updated: &'a T,
    ) -> Result<Option<(String, Vec<String>)>> {
        Self::shallow_diff_many_sql(std::iter::once((outdated, updated)))
    }

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    fn shallow_diff_many_sql<I>(entities: I) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = (&'a T, &'a T)> + 'a + Clone;
}


/// Defines a strategy to resolve the conflict, when the record to insert already exists.
pub enum DuplicateStrategy {
    /// Fail and return an error.
    Fail,
    /// Do not insert the record.
    Skip,
    /// Do not insert, but update the record instead.
    Update
}


/// Trait for insert. They work with entities.
/// This trait is implemented if keys of an entity are inserted too. This is typically the case for association tables.
/// Conflicts can happed if the keys already exist. A strategy must be provided to tell how to resolve the conflict.
pub trait Insert<'a, T: 'a> {
    /// Insert one struct, returns tuple with SQL statement and SQL params or error.
    fn insert_one_sql(entity: &'a T, strategy: DuplicateStrategy) -> Result<(String, Vec<String>)> {
        Ok(Self::insert_many_sql(std::iter::once(entity), strategy)?.unwrap())
    }
    /// Insert many structs, returns tuple with SQL statement and SQL params, none if no entities are provided or error.
    fn insert_many_sql<I>(entities: I,strategy: DuplicateStrategy) -> Result<Option<(String, Vec<String>)>>
    where
        I: IntoIterator<Item = &'a T> + 'a;
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

pub fn collection_delta_sql<'a, T>(
    outdated: &'a Vec<T>,
    updated: &'a Vec<T>,
) -> Result<(
    Option<(String, Vec<String>)>,
    Option<(String, Vec<String>)>,
    Option<(String, Vec<String>)>,
)>
where
    T: Update<'a, T> + 'a + Key + Delete<'a, T> + Insert<'a, T> 
    
{
    let mut insert: Vec<&T> = Vec::new();
    let mut diff: Vec<(&T, &T)> = Vec::new();
    let mut delete: Vec<T::Key> = Vec::new();
    let (mut ins, mut di, mut de) =
        collections_delta(std::iter::once((outdated, updated)))?;
    insert.append(&mut ins);
    diff.append(&mut di);
    delete.append(&mut de);

    let insert_sql = <T as Insert<T>>::insert_many_sql(insert, DuplicateStrategy::Fail)?;
    let diff_sql = <T as  Update<T>>::shallow_diff_many_sql(diff)?;
    let delete_sql = <T as  Delete<T>>::delete_many_sql(delete)?;
    Ok((insert_sql, diff_sql, delete_sql))
}

pub fn collections_delta<'a, I, T>(
    collections: I,
) ->  Result<(Vec<&'a T>, Vec<(&'a T, &'a T)>, Vec<T::Key>)>
where
    I: IntoIterator<Item = (&'a Vec<T>, &'a Vec<T>)> + 'a + Clone,
    T:  Update<'a, T> + Key + 'a +  Delete<'a, T>,
{
    let mut diff: Vec<(&T, &T)> = Vec::new(); // Vector with entities to diff
    let mut insert: Vec<&T> = Vec::new(); // Vector with entities to insert
    let mut delete: Vec<T::Key> = Vec::new(); // Vector with keys to delete

    for (previous_coll, current_coll) in collections {
        let mut previous_index: HashMap<T::Key, &T> = HashMap::new();
        for previous in previous_coll {
            // Build up index
            let k = Key::get_key(previous)?;
            previous_index.insert(k, &previous);
        }

        for current in current_coll {
            if previous_index.contains_key(&Key::get_key(current)?) {
                diff.push((
                    previous_index
                        .remove(&Key::get_key(current)?)
                        .unwrap(),
                    &current,
                ));
            } else {
                insert.push(&current);
            }
        }

        for (_k, v) in previous_index {
            delete.push(Key::get_key(v)?);
        }
    }

    Ok((insert, diff, delete))
}


