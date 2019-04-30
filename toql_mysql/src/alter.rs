
use mysql::error::Error;
use toql_core::error::ToqlError;


pub trait Alter<'a, T: 'a> {
    fn insert_one (entity: & T, conn: &mut mysql::Conn) -> Result<u64, ToqlError>;
    fn insert_many<I> (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> where I: Iterator<Item = &'a T>; 
    fn update_one (entity: & T, conn: &mut mysql::Conn) -> Result<u64, ToqlError>;
    fn update_many<I> (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> where I: Iterator<Item = &'a T>; 
    fn delete_one (entity: & T, conn: &mut mysql::Conn) -> Result<u64, ToqlError>;
    fn delete_many<I> (entities: I, conn: &mut mysql::Conn) -> Result<u64, ToqlError> where I: Iterator<Item = &'a T>; 
}

