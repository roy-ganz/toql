//! An iterator that maps entities into keys
/// This `struct` is created by the map_key method on Iterator. See its documentation for more.
use crate::keyed::Keyed;

pub struct MapKeyIter<I> {
    orig: I,
}

impl<I> Iterator for MapKeyIter<I>
where
    I: Iterator,
    I::Item: Keyed,
{
    type Item = <I::Item as Keyed>::Key;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.orig.next().map(|v| v.key())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.orig.size_hint()
    }
}
/// Takes an entity and turn it into a key.
/// This can also be used to create key predicates.
/// 
/// ### Examples
/// Basic usage (assume a TOQL derived User struct):
///
/// ```
///  let users = vec![User{id:5}, User{id:7}];
///  let keys = users.iter().map_key().collect::<Vec<_>>(); // Gives Vec<UserKey>
///  let predicate = users.iter().map_key().collect::<Query>();
///  assert(predicate.to_string, "(id eq 5;id eq 7)");
/// ```
pub fn map_key<I: Iterator>(xs: I) -> MapKeyIter<I> {
    MapKeyIter { orig: xs }
}

pub trait MapKey: Sized {
    fn map_key(self) -> MapKeyIter<Self>;
}

impl<I: Iterator> MapKey for I {
    fn map_key(self) -> MapKeyIter<Self> {
        map_key(self)
    }
}
