//! # Key trait
//!
//! The key trait is implemented for every Toql derived struct.
//! The most useful functions for library consumers are [get_key] and [set_key] to access the primary key of a struct.
//! Notice that these operations fail, if the fields that should hold the values are `None`.
//!

use std::borrow::Borrow;

use crate::sql::Sql;
use crate::{sql_arg::SqlArg, sql_expr::SqlExpr};

/// Trait to define key type of a Toql entity.
pub trait Keyed {
    /// Type of key. Composite keys are tuples.
    type Key: Eq + std::hash::Hash + crate::key::Key;

    /// Return value of the key for a given entity.
    fn try_get_key(&self) -> crate::error::Result<Self::Key>;

    /// Sets the key on a given entity.
    fn try_set_key(&mut self, key: Self::Key) -> crate::error::Result<()>;
}

/// Trait to define key type of a Toql entity.
pub trait KeyedSlice<K>
where
    K: Keyed,
{
    /// Return value of the key for a given entity.
    fn try_get_keys(&self) -> crate::error::Result<Vec<K::Key>>;
}

impl<K> KeyedSlice<K> for Vec<K>
where
    K: Keyed,
{
    /// Return value of the key for a given entity.
    fn try_get_keys(&self) -> crate::error::Result<Vec<K::Key>> {
        let mut keys = Vec::new();

        for k in self {
            keys.push(k.try_get_key()?);
        }

        Ok(keys)
    }
}

impl<K> KeyedSlice<K> for &[K]
where
    K: Keyed,
{
    /// Return value of the key for a given entity.
    fn try_get_keys(&self) -> crate::error::Result<Vec<K::Key>> {
        let mut keys = Vec::new();

        for k in *self {
            keys.push(k.try_get_key()?);
        }

        Ok(keys)
    }
}

pub fn keys<K: Keyed>(entities: &[K]) -> crate::error::Result<Vec<K::Key>> {
    let mut keys = Vec::with_capacity(entities.len());
    for e in entities {
        keys.push(e.try_get_key()?);
    }
    Ok(keys)
}

fn predicate_from_columns_with_alias_sql<K: Key, T, U>(
    keys: &[K],
    columns: &[T],
    sql_alias: Option<U>,
) -> Sql
where
    T: Borrow<str>,
    U: Borrow<str>,
{
    let mut params: Vec<SqlArg> = Vec::new();
    let mut predicate = String::new();

    if columns.len() == 1 {
        if let Some(ref a) = sql_alias {
            predicate.push_str(a.borrow());
            predicate.push('.');
        }
        predicate.push_str(columns.get(0).unwrap().borrow());
        predicate.push_str(" IN (");
        for key in keys {
            predicate.push_str(" ?, ");
            params.extend_from_slice(&K::params(key));
        }
        // Remove ' ,'
        predicate.pop();
        predicate.pop();
        predicate.push(')');
    } else {
        let mut single_predicate = String::new();
        for c in columns {
            if let Some(ref a) = sql_alias {
                single_predicate.push_str(a.borrow());
                single_predicate.push('.');
            }
            single_predicate.push_str(c.borrow());
            single_predicate.push_str(" = ? AND ");
        }
        // Remove ' AND '
        single_predicate.pop();
        single_predicate.pop();
        single_predicate.pop();
        single_predicate.pop();
        single_predicate.pop();

        predicate.push('(');

        for key in keys {
            predicate.push_str(&single_predicate);
            predicate.push_str(" OR ");
            params.extend_from_slice(&K::params(key));
        }
        // Remove ' OR '
        predicate.pop();
        predicate.pop();
        predicate.pop();
        predicate.pop();
        predicate.push(')');
    }

    Sql(predicate, params)
}

pub fn predicate_from_columns_sql<K: Key, T>(keys: &[K], aliased_columns: &[T]) -> Sql
where
    T: Borrow<str>,
{
    predicate_from_columns_with_alias_sql::<K, T, &str>(keys, aliased_columns, None)
}

/* pub fn predicate_sql_expr<K: Key, U>(keys: &[K], sql_alias: Option<U>) -> crate::sql_expr::SqlExpr
where
    U: Borrow<str>,
{
    predicate_from_columns_with_alias_sql::<K, _, U>(keys, &K::columns(), sql_alias)
} */

pub fn predicate_expr<K: Key>(key: K) -> SqlExpr {
    let columns = <K as Key>::columns();
    let mut params = key.params().into_iter();
    let mut expr = SqlExpr::new();

    for c in columns {
        if !expr.is_empty() {
            expr.push_literal(" AND ".to_string());
        }
        expr.push_self_alias();
        expr.push_literal(".");
        expr.push_literal(c);
        expr.push_literal(" = ".to_string());
        expr.push_arg(params.next().unwrap_or(SqlArg::Null()));
    }

    expr
}

pub fn predicate_sql<K: Key, U>(keys: &[K], sql_alias: Option<U>) -> Sql
where
    U: Borrow<str>,
{
    predicate_from_columns_with_alias_sql::<K, _, U>(keys, &K::columns(), sql_alias)
}
/*
pub fn predicate_sql<K :Key>(keys: &[K::Key], sql_alias: Option<&str>) -> (String, Vec<String>){

    let mut params: Vec<String> = Vec::new();
    let mut predicate = String::new();

    let columns = K::columns();
    if columns.len() == 1 {
            let mut predicate = String::from(columns.get(0).unwrap());
        predicate.push_str(" IN (");
        for key in keys {
            predicate.push_str(" ?, ");
            params.extend_from_slice(& K::params(key));
        }
        // Remove ' ,'
        predicate.pop();
        predicate.pop();
        predicate.push(')');
    } else {
        let mut single_predicate = String::new();
        for c in  &columns{
            if let Some(ref a)= sql_alias {
                single_predicate.push_str(a);
                single_predicate.push('.');
            }
                single_predicate.push_str(c);
                single_predicate.push_str(" = ? AND ");
            }
        // Remove ' AND '
        single_predicate.pop();
        single_predicate.pop();
        single_predicate.pop();
        single_predicate.pop();
        single_predicate.pop();

        predicate.push('(');

        for key in keys {
            predicate.push_str(&single_predicate);
            predicate.push_str(" OR ");
            params.extend_from_slice(& K::params(key));
        }
        // Remove ' OR '
        predicate.pop();
        predicate.pop();
        predicate.pop();
        predicate.pop();
        predicate.push(')');


    }
        (predicate, params)
}



 */

/// Trait to provide a partial key type of a Toql entity.
/// Only entities with composite keys can have a partial key.
/// Only the select_many function makes use of it (to select a merged collection from an entity).
pub trait PartialKey {
    type Key: Eq + std::hash::Hash;
}

/// Trait to provide the entity type for a key. This is only used
/// for ergonomics of the api.
pub trait Key {
    type Entity;

    /// Return primary key columns for a given entity.
    fn columns() -> Vec<String>;

    /// Return foreign key columns that match the primary keys for a given entity.
    /// This is only needed to merge entities.
    /// The names are calculated and do not necessarily match
    /// the actual foreign keys on the other table.
    /// The translation rules are (for snake case column format):
    ///
    /// | Type          | Guessing rule             | Example      |
    /// | --------------|---------------------------|---------------|
    /// | Normal fields |  tablename + normal field | `id` -> `user_id`, `access_code` -> `user_access_code` |
    /// | Joins         |  *No change* | `language_id` -> `language_id` |
    ///
    /// If the automatic generated names are not correct, the user is required to correct them by attributing
    /// the relevant field with
    ///  
    /// `#[toql( merge( columns( self = "id", other = "user_code")))]`
    ///
    fn default_inverse_columns() -> Vec<String>;

    /// Return key values as params. Useful to loop across a composite key.
    fn params(&self) -> Vec<SqlArg>;
}
/*
pub trait ToSqlPredicate{

    fn to_sql_predicate(&self, alias: &str) ->Sql;
    fn build_sql_predicate(&self, aliased_predicate: &str) -> Sql;
}

impl<T> ToSqlPredicate for T
where T: Key
{
    fn to_sql_predicate(&self, alias: &str) -> Sql {
         let mut predicate = String::new();
         let mut params = Vec::new();

         Self::columns().iter().zip(self.params()).for_each(|(col, par)| {
             predicate.push_str(alias);
             predicate.push('.');

             predicate.push_str(&col);
             predicate.push_str("= ? AND ");
             params.push(par);
            });
            predicate.pop();
            predicate.pop();
            predicate.pop();
            predicate.pop();
            predicate.pop();

            (predicate, params)
    }
     fn build_sql_predicate(&self, aliased_predicate: &str) -> Sql {

            // todo in debug check number of ? corresponds to params
            if cfg!(debug_assertions) {
                let expect = aliased_predicate.matches('?').count();
                if expect != self.params().len() {
                    panic!("Predicate `{}` does not have {} placeholders.",  aliased_predicate, expect);
                }
            }
            (aliased_predicate.to_string(), self.params())
     }
}

impl<T> ToSqlPredicate for &[T]
where T: ToSqlPredicate {

    fn to_sql_predicate(&self, alias: &str) -> Sql {

    let mut predicate = String::new();
         let mut params = Vec::new();

        for p in *self {
            let (pr, pa) = p.to_sql_predicate(alias);
            predicate.push_str(&pr);
            params.extend_from_slice(&pa);
            predicate.push_str(" OR ")
        }
        // Remove final " OR "
        predicate.pop();
        predicate.pop();
        predicate.pop();
        predicate.pop();

        (predicate, params)

    }
     fn build_sql_predicate(&self, aliased_predicate: &str) -> Sql {

        let mut predicate = String::new();
        let mut params = Vec::new();

           for p in *self {
              let (pr, pa) = p.build_sql_predicate(aliased_predicate);
              predicate.push_str(&pr);
              predicate.push_str(" OR ");
              params.extend_from_slice(&pa);
          }
        // Remove final " OR "
         predicate.pop();
         predicate.pop();
         predicate.pop();
         predicate.pop();

         (predicate, params)
     }
}



 */

/*

    /// Returns SQL predicate for collection.
    /// This may be overridded for simple primary keys that are build with IN(..)
    pub fn sql_predicate<'a, K, Q>(keys: &[Q], alias:&str) -> Sql
    where K: crate::key::Key, Q: 'a + Borrow<K>
    {
        let mut predicate = String::new();
         let mut params = Vec::new();

         for k in keys {
            K::columns().iter().zip(k.borrow().params()).for_each(|(col, par)| {
             predicate.push_str(alias);
             predicate.push('.');

             predicate.push_str(&col);
             predicate.push_str("= ? AND ");
             params.push(SqlArg::from(par));
            });
            predicate.pop();
            predicate.pop();
            predicate.pop();
            predicate.pop();
            predicate.pop();

            predicate.push_str(" OR ")
         }

         predicate.pop();
         predicate.pop();
         predicate.pop();
         predicate.pop();

         (predicate, params)
     }

     pub fn sql_expression<'a, K, Q>(keys: &[Q], sql:&str) -> Sql
    where K: crate::key::Key, Q: 'a + Borrow<K>
    {
         let mut predicate = String::new();
         let mut params = Vec::new();



          for k in keys {
              // TODO check number of params equal ? in sql
              predicate.push_str(sql);
              predicate.push_str(" OR ");
              params.extend_from_slice(&k.borrow().params());
          }

         predicate.pop();
         predicate.pop();
         predicate.pop();
         predicate.pop();

         (predicate, params)

    }


*/
/* pub fn key_translation(keys: &HashSet<u64>, mut id: u64) -> HashMap<u64, u64>
 where
{

    let mut translation = HashMap::new();
    for k in keys {
        translation.insert( *k, id);
        id += 1;
    }
    translation
} */

/*
pub fn default_inverse_predicate<K>(key: K, alias: &str) -> (String, Vec<String>)
where K:Key
{
        let mut predicate= String::from("(");

        for c in Key::default_inverse_columns() {
            predicate.push_str(alias);
            predicate.push('.');
            predicate.push_str(&c);
            predicate.push_str(" = ? AND ");
        }
    predicate.pop();
    predicate.pop();
    predicate.pop();
    predicate.pop();
    predicate.push(')');
    (predicate, k.params())

} */

pub fn params<K>(keys: &[K]) -> Vec<SqlArg>
where
    K: Key,
{
    let mut params = Vec::new();
    for k in keys {
        params.extend_from_slice(&k.params());
    }
    params
}
