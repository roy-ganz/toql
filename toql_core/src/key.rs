//! # Key trait
//!
//! The key trait is used for generic operations that need to build an index for faster processing.
//! The merge and diff functions need indexes.
//!

/// Trait to define key type of a Toql entity.
pub trait Key {
    /// Type of key. Composite keys are tuples.
    type Key: Eq + std::hash::Hash;

    /// Return value of the key for a given entity.
    fn get_key(&self) -> crate::error::Result<Self::Key>;

    /// Sets the key on a given entity.
    fn set_key(&mut self, key: Self::Key) -> crate::error::Result<()>;

    /// Return primary key columns for a given entity.
    fn columns() -> Vec<String>;

    /// Return foreign key columns for a given entity.
    /// The names are calculated and do not necessarily match
    /// the actual foreign keys on the other table. 
    /// The translation rules are (for snake case):
    /// - normal fields -> tablename + normal field
    ///   id -> user_id
    ///   access_code -> user_access_code
    /// - joins -> no change
    ///   language_id -> language_id
    fn default_inverse_columns() -> Vec<String>;

    // Return key as params for a given entity.
    fn params(key: &Self::Key) -> Vec<String>;
}


pub fn keys<K :Key>(entities: &[K]) ->  crate::error::Result<Vec<K::Key>>{
    let mut keys = Vec::with_capacity(entities.len());
    for e in entities {
        keys.push(e.get_key()?);
    }
    Ok(keys)
}



