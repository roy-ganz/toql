
use crate::error::ToqlError;

pub trait Indelup<'a, T: 'a> {
    fn insert_one_sql (entity: & T) -> Result<(String, Vec<String>), ToqlError>;
    fn insert_many_sql<I> (entities: I) -> Result<(String, Vec<String>), ToqlError> where I: IntoIterator<Item = &'a T> + 'a; 
    fn delete_one_sql (entity: & T) -> Result<(String, Vec<String>), ToqlError>;
    fn delete_many_sql<I> (entities: I) -> Result<(String, Vec<String>), ToqlError> where I: IntoIterator<Item = &'a T> + 'a; 
    fn update_one_sql (entity: & T) -> Result<(String, Vec<String>), ToqlError>;
    fn update_many_sql<I> (entities: I) -> Result<(String, Vec<String>), ToqlError> where I: IntoIterator<Item = &'a T> + 'a + Clone;
}
