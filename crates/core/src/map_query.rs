use std::marker::PhantomData;

use crate::key::Key;
use crate::query::Query;
//use crate::to_query::ToQuery;

pub struct MapQueryIter<I, T> {
    orig: I,
    phantom: PhantomData<T>,
}

impl<I, T> Iterator for MapQueryIter<I, T>
where
    I: Iterator,
    I::Item: Into<Query<T>>,
{
    type Item = Query<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.orig.next().map(|v| v.into())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.orig.size_hint()
    }
}

pub fn map_query<I: Iterator, T>(xs: I) -> MapQueryIter<I, T> {
    MapQueryIter {
        orig: xs,
        phantom: PhantomData,
    }
}

pub trait MapQuery: Sized {
    fn map_query<T>(self) -> MapQueryIter<Self, T>;
}

impl<I: Iterator> MapQuery for I {
    fn map_query<T>(self) -> MapQueryIter<Self, T> {
        map_query(self)
    }
}

// Allows to collect different queries in a query (concatenation is and)
impl<'a, T> std::iter::FromIterator<Query<T>> for Query<T> {
    fn from_iter<I: IntoIterator<Item = Query<T>>>(iter: I) -> Query<T> {
        let mut q: Query<T> = Query::new();
        for i in iter {
            q = q.and(i);
        }
        q
    }
}

// Allows to collect different keys in a query (concatenation is or)
impl<T, K> std::iter::FromIterator<K> for Query<T>
where
    K: Key<Entity = T> + Into<Query<T>>,
{
    fn from_iter<I: IntoIterator<Item = K>>(iter: I) -> Query<T> {
        let mut q: Query<T> = Query::new();
        let mut count = 0;
        for k in iter {
            if count < 2 {
                count += 1}
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
/* impl<'a, T, K> std::iter::FromIterator<&'a K> for Query<T>
where K: Key<Entity=T> + ToQuery<T> + 'a
{
 fn from_iter<I: IntoIterator<Item =&'a K>>(iter: I) -> Query<T> {
      let mut q :Query<T> = Query::new();
        for k in iter {
            q = q.or(ToQuery::to_query(k) );
        }
        q
    }
} */
/*impl<'a, T, K> std::iter::FromIterator<K> for Query<T>
where K: std::borrow::Borrow<K>,
K: Key<Entity=T> + ToQuery<T> + 'a
{
 fn from_iter<I: IntoIterator<Item =K>>(iter: I) -> Query<T> {
      let mut q :Query<T> = Query::new();
        for k in iter {
            q = q.or(ToQuery::to_query(k.borrow()) );
        }
        q
    }
} */
