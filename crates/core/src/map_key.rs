use crate::keyed::Keyed;

pub struct MapKeyIter<I> {
    orig: I
}

impl<I> Iterator for MapKeyIter<I> where I: Iterator, I::Item : Keyed
{
    type Item = <I::Item as Keyed>::Key;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.orig.next().map(|v| v.key() )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.orig.size_hint() 
    }
}

pub fn map_key<I: Iterator>(xs: I) -> MapKeyIter<I> {
    MapKeyIter { orig: xs }
}

pub trait MapKey: Sized {
    fn map_key(self) -> MapKeyIter<Self>;
}

impl <I: Iterator> MapKey for I {
    fn map_key(self) -> MapKeyIter<Self> {
        map_key(self)
    }
}
