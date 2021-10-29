//! An iterator that maps entities into keys.

use crate::keyed::Keyed;

/// This `struct` is created by the [map_key] method on Iterator. See its documentation for more.
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
/// Takes a list of entities and turn them into keys.
/// This can also be used to create key predicates.
///
/// ### Example
/// - Basic collection of keys
/// - Building a key predicate for a Toql query
///
/// ```rust, ignore
/// use toql_derive::Toql;
/// use toql_core::{map_key::MapKey, query::Query};
///
/// #[derive(Toql)]
/// struct User {
///     #[toql(key)]
///     id: u64,
///     name: String
/// }
///
///  let users = vec![User{id:5, name: "Joe".to_string()}, User{id:7, name: "Sue".to_string()}];
///  let keys = users.iter().map_key().collect::<Vec<_>>(); // Returns Vec<UserKey>
///  let predicate = users.iter().map_key().collect::<Query>(); // Build query
///
///  assert_eq!(predicate.to_string, "(id eq 5;id eq 7)");
/// ```
/// Notice that when keys are be collected into a [Query](crate::query::Query) the
/// predicates are concatenated with OR.
pub fn map_key<I: Iterator>(xs: I) -> MapKeyIter<I> {
    MapKeyIter { orig: xs }
}

/// An iterator trait to turn entities into keys.
pub trait MapKey: Sized {
    fn map_key(self) -> MapKeyIter<Self>;
}

impl<I: Iterator> MapKey for I {
    fn map_key(self) -> MapKeyIter<Self> {
        map_key(self)
    }
}
