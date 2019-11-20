
use toql_core::error::Result;

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