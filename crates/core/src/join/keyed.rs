
use super::Join;
use crate::error::Result;
use crate::key::Keyed;

impl<E> Keyed for Join<E>
where
    E: Keyed,
    E::Key: Clone,
{
    type Key = E::Key;
    fn try_get_key(&self) -> Result<E::Key> {
        match self {
            Join::Key(k) => Ok(k.clone()),
            Join::Entity(e) => e.try_get_key(),
        }
    }
    fn try_set_key(&mut self, key: E::Key) -> Result<()> {
        match self {
            Join::Key(_) => {
                *self = Join::Key(key);
                Ok(())
            }
            Join::Entity(e) => e.try_set_key(key),
        }
    }
}


