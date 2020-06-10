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

use crate::alias_translator::AliasTranslator;
use crate::alias::AliasFormat;
use crate::sql_mapper_registry::SqlMapperRegistry;
use crate::error::ToqlError;
use crate::key::Keyed;
use core::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::result::Result;
use crate::sql_mapper::mapped::Mapped;
use crate::sql_mapper::SqlMapper;


use crate::sql::Sql;



/* /// Trait for delete functions (They work with entity keys).
pub trait Delete<'a, T: crate::key::Keyed + 'a> {
    type Error;
    /// Delete one structs, returns tuple with SQL statement and SQL params or error.
    fn delete_one_sql(
        key: T::Key,
        roles: &HashSet<String>,
    ) -> Result<Sql, Self::Error>
    where
        T: crate::key::Keyed + crate::sql_mapper::Mapped +  'a,
    {
        Ok(Self::delete_many_sql(crate::key::sql_predicate(&[key], &<T as  crate::sql_mapper::Mapped>::table_alias()), roles)?.unwrap())
    }
    /// Delete many structs, returns tuple with SQL statement and SQL params or error.
    fn delete_many_sql(
        predicate: Sql,
        roles: &HashSet<String>,
    ) -> Result<Option<Sql>, Self::Error>;
} */

/// Trait for update. They work with entities
pub trait UpdateSql {
    
    fn update_one_sql(
        &self,
        roles: &HashSet<String>,
    ) -> Result<Option<Sql>, ToqlError> {
        Self::update_many_sql(&[self], roles)
    }
    /// Update many structs, returns tuple with SQL statement and SQL params or error.
    fn update_many_sql<Q: Borrow<Self>>(
        entities: &[Q],
        roles: &HashSet<String>,
    ) -> Result<Option<Sql>, ToqlError>;
}

/// Trait for update. They work with entities
pub trait DiffSql {
   
    /// Update difference of two structs, given as tuple (old, new), returns a vectro with SQL statements and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    /// Because merged structs are also considered, the returned SQL statements, can be insert, update and delete statements.
    fn full_diff_one_sql<Q: Borrow<Self>>(
        outdated: Q,
        updated: Q,
        roles: &HashSet<String>,
        sql_mapper: &SqlMapper
    ) -> Result<Vec<Sql>, ToqlError> {
        Ok(Self::full_diff_many_sql(&[(outdated, updated)], roles, sql_mapper)?.unwrap())
    }

    /// Update difference of two structs, given as tuple (old, new), returns tuples with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs and merged structs.
    /// To exclude any fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    fn full_diff_many_sql<Q: Borrow<Self>>(
        entities: &[(Q, Q)],
        roles: &HashSet<String>,
        sql_mapper: &SqlMapper
    ) -> Result<Option<Vec<Sql>>,ToqlError>;

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup`  or set selectable fields to None in updated entity.
    fn diff_one_sql<Q: Borrow<Self>>(
        outdated: Q,
        updated: Q,
        roles: &HashSet<String>,
    ) -> Result<Option<Sql>, ToqlError> {
        Self::diff_many_sql(&[(outdated, updated)], roles)
    }

    /// Update difference of two structs, given as tuple (old, new), returns tuple with SQL statement and SQL params or error.
    /// This includes foreign keys of joined structs, but excludes merged structs
    /// To exclude any other fields annotate them with `skip_delup` or set selectable fields to None in updated entity.
    fn diff_many_sql<Q: Borrow<Self>>(
        entities: &[(Q, Q)],
        roles: &HashSet<String>,
    ) -> Result<Option<Sql>, ToqlError>;
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
pub trait InsertSql {
    

    /// Insert one struct, returns tuple with SQL statement and SQL params or error.
    fn insert_one_sql(
       &self,
        //strategy: DuplicateStrategy,
        roles: &HashSet<String>,
        modifier: &str,
        extra: &str,
    ) -> Result<Sql, ToqlError> {
        Ok(Self::insert_many_sql(&[self], roles, modifier, extra)?.unwrap())
    }
    /// Insert many structs, returns tuple with SQL statement and SQL params, none if no entities are provided or error.
    fn insert_many_sql<Q: Borrow<Self>>(
        entities: &[Q],
       // strategy: DuplicateStrategy,
        roles: &HashSet<String>,
        modifier: &str,
        extra: &str,
    ) -> Result<Option<Sql>, ToqlError>;
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
    outdated: &'a [T],
    updated: &'a [T],
    roles: HashSet<String>,
    sql_mapper_registry: &SqlMapperRegistry,
    format: AliasFormat
) -> Result<
    (
        Option<Sql>,
        Option<Sql>,
        Option<Sql>,
    ),
    ToqlError,
>
where
    T: 'a + Keyed + Mapped + InsertSql + UpdateSql + DiffSql,
    <T as Keyed>::Key: crate::to_query::ToQuery<T>
{
    use  crate::sql_builder::SqlBuilder;
   
    let mut insert: Vec<&T> = Vec::new();
    let mut diff: Vec<(&'a T, &'a T)> = Vec::new();
    let mut delete: Vec<T::Key> = Vec::new();
    let (mut ins, mut di, mut de) = collections_delta::<T>(&vec![(outdated, updated)])?;
    insert.append(&mut ins);
    diff.append(&mut di);
    delete.append(&mut de);

    let insert_sql = <T as InsertSql>::insert_many_sql(insert.as_slice(), &roles, "", "")?;
    let diff_sql = <T as DiffSql>::diff_many_sql(&diff, &roles)?;

     
    
     let delete_sql = if delete.is_empty() {None } else {
         Some(SqlBuilder::new( &<T as Mapped>::table_name(), &sql_mapper_registry).with_roles(roles)
         .build_delete_sql(&crate::to_query::ToQuery::slice_to_query(&delete), "", "", format)?)
     };

    Ok((insert_sql, diff_sql, delete_sql))
}

pub fn collections_delta<'a, T>(
    collections: &[(&'a [T], &'a [T])],
) -> Result<(Vec<&'a T>, Vec<(&'a T, &'a T)>, Vec<T::Key>), ToqlError>
where
    T: 'a + Keyed,
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
