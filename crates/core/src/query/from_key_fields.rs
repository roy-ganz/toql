
use crate::{
    query::{field::Field, Query},
    key_fields::KeyFields
};

impl<T> From<T> for Query<T::Entity> where T: KeyFields{

    fn from(fields:T) -> Query<T::Entity> {
        let mut query = Query::new();
        let args = fields.params();
        let fs = T::fields();
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
