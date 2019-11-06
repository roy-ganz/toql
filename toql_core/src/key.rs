//! # Key trait
//! 
//! The key trait is used for generic operations that need to build an index for faster processing.
//! The merge and diff functions need indexes.
//!



/// Trait to define key type of a Toql entity.
pub trait Key {

    /// Type of key. Composite keys are tuples.
   type Key : Eq + std::hash::Hash;

   /// Return value of the key for a given entity.
   fn get_key(&self) -> crate::error::Result<Self::Key>;

   /// Sets the key on a given entity. 
   fn set_key(&mut self, key: Self::Key) -> crate::error::Result<()>;

   // Return key columns for a given entity.
   fn columns() -> Vec<String>;

   // Return key as params for a given entity.
   fn params(key: &Self::Key) -> Vec<String>;
 
}
