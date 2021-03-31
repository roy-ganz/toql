
use super::Join;
use crate::keyed::{Keyed, KeyedMut};

impl<T> Keyed for Join<T>
where
    T: Keyed,
    T::Key: Clone,
{
    type Key = T::Key;
    fn key(&self) -> T::Key {
        match self {
            Join::Key(k) => k.clone(),
            Join::Entity(e) => e.key(),
        }
    }
   
}
impl<T> Keyed for &Join<T>
where
    T: Keyed,
    T::Key: Clone,
{
    type Key = T::Key;
    fn key(&self) -> T::Key {
        match self {
            Join::Key(k) => k.clone(),
            Join::Entity(e) => e.key(),
        }
    }
   
}
impl<T> KeyedMut for Join<T>
where
    T: KeyedMut,
    T::Key: Clone,
{
    fn set_key(&mut self, key: T::Key) {
        match self {
            Join::Key(_) => {
                *self = Join::Key(key);
               
            }
            Join::Entity(e) => e.set_key(key),
        }
    }
}



