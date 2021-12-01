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

#[cfg(test)]
mod test {
    use super::Query;
    use crate::{key::Key, key_fields::KeyFields, query::field::Field};

    #[test]
    /// Queries are concatenated with AND, no parenthesis are added automatically
    fn from_query() {
        struct User;

        let qa = Query::new().and(Field::from("a").eq(1));
        let qb = Query::new().and(Field::from("b").eq(3));
        let q: Query<User> = vec![qa, qb].into_iter().collect();
        assert_eq!(q.to_string(), "a EQ 1,b EQ 3");

        // No parenthesis are added automatically
        let qa = Query::new()
            .and(Field::from("a").eq(1))
            .or(Field::from("a").eq(2));
        let qb = Query::new().and(Field::from("b").eq(3));
        let q: Query<User> = vec![qa, qb].into_iter().collect();
        assert_eq!(q.to_string(), "a EQ 1;a EQ 2,b EQ 3");

        // User must add parenhesis
        let qa = Query::new()
            .and(Field::from("a").eq(1))
            .or(Field::from("a").eq(2))
            .parenthesize();
        let qb = Query::new().and(Field::from("b").eq(3));
        let q: Query<User> = vec![qa, qb].into_iter().collect();
        assert_eq!(q.to_string(), "(a EQ 1;a EQ 2),b EQ 3");
    }

    #[test]
    /// Keys are concatenated with OR, parenthesis are added
    fn from_key() {
        use crate::sql_arg::SqlArg;
        struct User {}

        struct UserKey {
            id: u64,
        }

        impl Key for UserKey {
            type Entity = User;

            fn columns() -> Vec<String> {
                vec!["id".to_string()]
            }

            fn default_inverse_columns() -> Vec<String> {
                vec!["user_id".to_string()]
            }

            fn params(&self) -> Vec<SqlArg> {
                vec![SqlArg::from(self.id)]
            }
        }
        impl KeyFields for UserKey {
            type Entity = User;

            fn fields() -> Vec<String> {
                vec!["id".to_string()]
            }

            fn params(&self) -> Vec<SqlArg> {
                vec![SqlArg::from(self.id)]
            }
        }

        // Multiple keys, no parentheses
        let k1 = UserKey { id: 1 };

        let q: Query<User> = vec![k1].into_iter().collect();
        assert_eq!(q.to_string(), "id EQ 1");

        // Multiple keys, with parentheses
        let k1 = UserKey { id: 1 };
        let k2 = UserKey { id: 2 };

        let q: Query<User> = vec![k1, k2].into_iter().collect();
        assert_eq!(q.to_string(), "(id EQ 1;id EQ 2)");

        // No smart optimisation, such as `id IN 1 2 3`
        let k1 = UserKey { id: 1 };
        let k2 = UserKey { id: 2 };
        let k3 = UserKey { id: 3 };

        let q: Query<User> = vec![k1, k2, k3].into_iter().collect();
        assert_eq!(q.to_string(), "(id EQ 1;id EQ 2;id EQ 3)");

        // Satisfy line coverage
        let k1 = UserKey { id: 1 };
        assert_eq!(UserKey::columns(), vec!["id".to_string()]);
        assert_eq!(
            UserKey::default_inverse_columns(),
            vec!["user_id".to_string()]
        );
        assert_eq!(<UserKey as Key>::params(&k1), vec![SqlArg::U64(1)]);
    }
}
