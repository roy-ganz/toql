
use crate::query::Query;
use std::borrow::Borrow;

/// Trait for keys to be converted to foreign queries. 
pub trait ToQuery<T> {
    fn to_query(&self) -> Query<T>;
        
    

    /// Turn a slice of keys into a query
    /// Due to orphan rule, trait cannot be implemented on array
    /// Simple keys may implement this with IN operator, default 
    /// implementation may use defaul implementation with or
    fn slice_to_query(entities: &[Self]) -> Query<T> 
    where Self: Sized
    {
        let mut q = crate::query::Query::<T>::new();
        for e in entities {
            q = q.or_parentized(e.to_query());   
        };
        q
    }
}


/// Trait for keys to be converted to foreign queries. 
pub trait ToForeignQuery {
  
    fn to_foreign_query<T>(&self, path:&str) -> Query<T>;

    fn slice_to_foreign_query<T>(entities: &[Self], path:&str) -> Query<T>
    where Self: Sized
    {
        let mut q = crate::query::Query::<T>::new();
        for e in entities {
            q = q.or_parentized(e.to_foreign_query::<T>(path));   
        };
        q
    }
}

