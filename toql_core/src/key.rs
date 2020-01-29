//! # Key trait
//!
//! The key trait is used for generic operations that need to build an index for faster processing.
//! The merge and diff functions need indexes.
//!

use std::borrow::Borrow;

/// Trait to define key type of a Toql entity.
pub trait Key {
    /// Type of key. Composite keys are tuples.
    type Key: Eq + std::hash::Hash;

    /// Return value of the key for a given entity.
    fn get_key(&self) -> crate::error::Result<Self::Key>;

    /// Sets the key on a given entity.
    fn set_key(&mut self, key: Self::Key) -> crate::error::Result<()>;

    /// Return primary key columns for a given entity.
    fn columns() -> Vec<String>;

    /// Return foreign key columns for a given entity.
    /// The names are calculated and do not necessarily match
    /// the actual foreign keys on the other table. 
    /// The translation rules are (for snake case):
    /// - normal fields -> tablename + normal field
    ///   id -> user_id
    ///   access_code -> user_access_code
    /// - joins -> no change
    ///   language_id -> language_id
    fn default_inverse_columns() -> Vec<String>;

    // Return key as params for a given entity.
    fn params(key: &Self::Key) -> Vec<String>;
}


pub fn keys<K :Key>(entities: &[K]) ->  crate::error::Result<Vec<K::Key>>{
    let mut keys = Vec::with_capacity(entities.len());
    for e in entities {
        keys.push(e.get_key()?);
    }
    Ok(keys)
}

fn predicate_from_columns_with_alias_sql<K :Key, T, U>(keys: &[K::Key],columns: &[T], sql_alias: Option<U>) -> (String, Vec<String>)
where T: Borrow<str>, U:Borrow<str>
{
        let mut params: Vec<String> = Vec::new();
        let mut predicate = String::new();


        
          if columns.len() == 1 {
                if let Some(ref a)= sql_alias {
                    predicate.push_str(a.borrow());
                    predicate.push('.');
                }
               predicate.push_str(columns.get(0).unwrap().borrow());
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
            for c in  columns{
                if let Some(ref a)= sql_alias {
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

pub fn predicate_from_columns_sql<K :Key, T>(keys: &[K::Key],aliased_columns: &[T]) -> (String, Vec<String>)
where T: Borrow<str>{
    predicate_from_columns_with_alias_sql::<K,T,&str>(keys, aliased_columns,None)
}

pub fn predicate_sql<K :Key, U>(keys: &[K::Key],sql_alias: Option<U>) -> (String, Vec<String>)
where U: Borrow<str>
{
    predicate_from_columns_with_alias_sql::<K,_,U>(keys, &K::columns(),sql_alias)
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