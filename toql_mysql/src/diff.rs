/// Update difference of two collections
/// Compares multiple tuples with outdated / current collections and builds insert / update / delete statements
/// to save the changes in a database.
///
///
/// Returns three tuples for insert / update / delete, each containing the SQL statement and parameters.
use std::collections::HashMap;
use toql_core::mutate::{Update, Delete};
use toql_core::key::Key;
use crate::insert::{Insert, DuplicateStrategy};
use toql_core::error::Result;

pub fn collection_delta_sql<'a, T>(
    outdated: &'a Vec<T>,
    updated: &'a Vec<T>,
) -> Result<(
    Option<(String, Vec<String>)>,
    Option<(String, Vec<String>)>,
    Option<(String, Vec<String>)>,
)>
where
    T: Update<'a, T> + 'a + Key + Delete<'a, T> + Insert<'a, T> 
    
{
    let mut insert: Vec<&T> = Vec::new();
    let mut diff: Vec<(&T, &T)> = Vec::new();
    let mut delete: Vec<T::Key> = Vec::new();
    let (mut ins, mut di, mut de) =
        crate::diff::collections_delta(std::iter::once((outdated, updated)))?;
    insert.append(&mut ins);
    diff.append(&mut di);
    delete.append(&mut de);

    let insert_sql = <T as Insert<T>>::insert_many_sql(insert, DuplicateStrategy::Fail)?;
    let diff_sql = <T as  Update<T>>::shallow_diff_many_sql(diff)?;
    let delete_sql = <T as  Delete<T>>::delete_many_sql(delete)?;
    Ok((insert_sql, diff_sql, delete_sql))
}

pub fn collections_delta<'a, I, T>(
    collections: I,
) ->  Result<(Vec<&'a T>, Vec<(&'a T, &'a T)>, Vec<T::Key>)>
where
    I: IntoIterator<Item = (&'a Vec<T>, &'a Vec<T>)> + 'a + Clone,
    T:  Update<'a, T> + Key + 'a +  Delete<'a, T>,
{
    let mut diff: Vec<(&T, &T)> = Vec::new(); // Vector with entities to diff
    let mut insert: Vec<&T> = Vec::new(); // Vector with entities to insert
    let mut delete: Vec<T::Key> = Vec::new(); // Vector with keys to delete

    for (previous_coll, current_coll) in collections {
        let mut previous_index: HashMap<T::Key, &T> = HashMap::new();
        for previous in previous_coll {
            // Build up index
            let k = Key::get_key(previous)?;
            previous_index.insert(k, &previous);
        }

        for current in current_coll {
            if previous_index.contains_key(&Key::get_key(current)?) {
                diff.push((
                    previous_index
                        .remove(&Key::get_key(current)?)
                        .unwrap(),
                    &current,
                ));
            } else {
                insert.push(&current);
            }
        }

        for (_k, v) in previous_index {
            delete.push(Key::get_key(v)?);
        }
    }

    Ok((insert, diff, delete))
}
