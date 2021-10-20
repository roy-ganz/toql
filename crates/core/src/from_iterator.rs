//! Collect keys and structs that implement `Into<Query>` into [Query](crate::query::Query).
use crate::key::Key;
use crate::query::Query;

/// Allows to collect different queries in a query (concatenation is and)
impl<'a, T> std::iter::FromIterator<Query<T>> for Query<T> {
    fn from_iter<I: IntoIterator<Item = Query<T>>>(iter: I) -> Query<T> {
        let mut q: Query<T> = Query::new();
        for i in iter {
            q = q.and(i);
        }
        q
    }
}

/// Allows to collect different keys in a query (concatenation is or)
impl<T, K> std::iter::FromIterator<K> for Query<T>
where
    K: Key<Entity = T> + Into<Query<T>>,
{
    fn from_iter<I: IntoIterator<Item = K>>(iter: I) -> Query<T> {
        let mut q: Query<T> = Query::new();
        let mut count = 0;
        for k in iter {
            if count < 2 {
                count += 1
            }
            q = q.or(k.into());
        }
        // Only parenthesize if there is more than one key
        if count > 1 {
            q.parenthesize()
        } else {
            q
        }
    }
}
