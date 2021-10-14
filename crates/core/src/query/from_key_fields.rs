use crate::{
    key_fields::KeyFields,
    query::{field::Field, Query},
};

impl<T> From<T> for Query<T::Entity>
where
    T: KeyFields,
{
    fn from(fields: T) -> Query<T::Entity> {
        let mut query = Query::new();
        let params = fields.params();
        let fs = T::fields();
        // Composite key
        if params.len() > 1 {
            let mut q: Query<T::Entity> = Query::new();
            for (f, a) in fs.iter().zip(params) {
                q = q.and(Field::from(f).eq(a));
            }
            query = query.and_parentized(q);
        } else {
            for (f, a) in fs.iter().zip(params) {
                query = query.and(Field::from(f).eq(a));
            }
        }
        query
    }
}
