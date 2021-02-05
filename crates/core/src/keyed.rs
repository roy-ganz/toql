
/// Trait to define key type of a Toql entity.
pub trait Keyed {
    /// Type of key. Composite keys are tuples.
    type Key: Eq + std::hash::Hash + crate::key::Key;

    /// Return value of the key for a given entity.
    fn key(&self) -> Self::Key;
    // fn set_key(&mut self, key: Self::Key);

}

pub trait KeyedMut : Keyed{
   
    /// Return value of the key for a given entity.
    fn set_key(&mut self, key: Self::Key);
}






