
use super::Join;
use crate::error::Result;
use crate::key::Keyed;

impl<T> Keyed for Join<T>
where
    T: Keyed,
    T::Key: Clone,
{
    type Key = T::Key;
    fn try_get_key(&self) -> Result<T::Key> {
        match self {
            Join::Key(k) => Ok(k.clone()),
            Join::Entity(e) => e.try_get_key(),
        }
    }
    fn try_set_key(&mut self, key: T::Key) -> Result<()> {
        match self {
            Join::Key(_) => {
                *self = Join::Key(key);
                Ok(())
            }
            Join::Entity(e) => e.try_set_key(key),
        }
    }
}


