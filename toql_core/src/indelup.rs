
use crate::error::Result;

pub trait Indelup<'a, T: 'a> {
    fn insert_one_sql (entity: & T) -> Result<(String, Vec<String>)>;
    fn insert_many_sql<I> (entities: I) -> Result<(String, Vec<String>)> where I: IntoIterator<Item = &'a T> + 'a; 
    fn delete_one_sql (entity: & T) -> Result<(String, Vec<String>)>;
    fn delete_many_sql<I> (entities: I) -> Result<(String, Vec<String>)> where I: IntoIterator<Item = &'a T> + 'a; 
    fn update_one_sql (entity: & T) -> Result<(String, Vec<String>)>;
    fn update_many_sql<I> (entities: I) -> Result<(String, Vec<String>)> where I: IntoIterator<Item = &'a T> + 'a + Clone;
}
