use std::marker::PhantomData;

use crate::to_query::ToQuery;
use crate::query::Query;
use crate::key::Key;

pub struct MapQueryIter<I, T> {
    orig: I,
    phantom : PhantomData<T>
}

impl<I, T> Iterator for MapQueryIter<I, T> 
where I: Iterator, 
    I::Item : ToQuery<T>,
{
   type Item = Query<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.orig.next().map(|v| v.to_query() )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.orig.size_hint() 
    }
}

pub fn map_query<I: Iterator, T>(xs: I) -> MapQueryIter<I, T> {
    MapQueryIter { orig: xs, phantom: PhantomData }
}

pub trait MapQuery: Sized {
    fn map_query<T>(self) -> MapQueryIter<Self, T>;
}

impl <I: Iterator> MapQuery for I {
    fn map_query<T>(self) -> MapQueryIter<Self, T> {
        map_query(self)
    }
}

// Allows to collect different queries in a query
impl<'a, T> std::iter::FromIterator<Query<T>> for Query<T> {
 fn from_iter<I: IntoIterator<Item = Query<T>>>(iter: I) -> Query<T> {
      let mut q :Query<T> = Query::new();
      for i in iter{
        q = q.and(i);
      }
        q
    }
}

// Allows to collect different keys in a query
impl<'a, T, K> std::iter::FromIterator<K> for Query<T> 
where K: Key<Entity=T> + ToQuery<T>
{
 fn from_iter<I: IntoIterator<Item =K>>(iter: I) -> Query<T> {
      let mut q :Query<T> = Query::new();
        for k in iter {
            q = q.and(ToQuery::to_query(&k) );
        }
        q
    }
}