use super::KeyFields;
use crate::{
    query::{field::Field, Query},
    to_query::ToQuery,
};

impl<T> ToQuery<T::Entity> for T
where
    T: KeyFields,
{
    fn to_query(&self) -> Query<T::Entity> {
        let mut query = Query::new();
        let args = self.params();
        let fs = Self::fields();
        // Komposite key
        if args.len() > 1 {
            let mut q: Query<T::Entity> = Query::new();
            for (f, a) in fs.iter().zip(args) {
                q = q.and(Field::from(f).eq(a));
            }
            query = query.or_parentized(q);
        } else {
            for (f, a) in fs.iter().zip(args) {
                query = query.and(Field::from(f).eq(a));
            }
        }
        query
    }
}
