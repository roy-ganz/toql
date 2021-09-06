use async_trait::async_trait;
use std::borrow::{Borrow, BorrowMut};


 use std::result::Result;
 use crate::error::ToqlError;
 use crate::from_row::FromRow;
 use crate::keyed::Keyed;
 use crate::key::Key;
 use crate::page::Page;
 use crate::query::Query;
 use crate::to_query::ToQuery;


pub mod count;
pub mod delete;
pub mod fields;
pub mod insert;
pub mod load;
pub mod paths;
pub mod update;



use insert::Insert;
use update::Update;
use load::Load;
use count::Count;
use delete::Delete;
use fields::Fields;
use paths::Paths;

/// This trait provides a common interface for all database backends. 
/// That means every database provider must implement this trait.
/// This ensures that the user of Toql can easily switch backends with less hussle.
/// The trait can also be used to write database independend code. This however requires more trait bounds
/// and is usually less ergonomic than writing for a specific database.
/// The following code shows the function signature to load an entity `MyUser` from a any database:
/// ```
/// use toql::prelude::{ToqlError, ToqlApi, Load, FromRow} 
/// async fn load_user<R, E, A>(toql: &mut A) -> std::result::Result<Vec<MyUser>, MyError>
/// where A: ToqlApi<Row=R, Error = E>, E: From<ToqlError>, MyUser: Load<R, E>, <MyUser as Keyed>::Key: FromRow<R, E>, MyError: From<E>
/// {
///        let users = toql.load_many().await?;
///        Ok(users)
///  }
/// ```
#[async_trait]
pub trait ToqlApi {
    type Row;
    type Error;

    async fn insert_one<T>(&mut self, entity: &mut T, paths: Paths) -> Result<(), Self::Error>
    where T: Insert;

    async fn insert_many<T, Q>(&mut self, entities: &mut [Q], paths: Paths) -> Result<(), Self::Error>
    where  T: Insert, Q: BorrowMut<T> + Send;

    async fn update_one<T>(&mut self, entity: &mut T, fields: Fields) -> Result<(), Self::Error>
    where T: Update;

    async fn update_many<T, Q>(&mut self, entities: &mut [Q], fields: Fields) -> Result<(), Self::Error>
    where T: Update, Q: BorrowMut<T> + Send;

    async fn load_one<T, B>(&mut self, query: B) -> Result<T, Self::Error>
    where T: Load<Self::Row, Self::Error>, B: Borrow<Query<T>> + Send + Sync, <T as Keyed>::Key: FromRow<Self::Row, Self::Error>,
    <Self as ToqlApi>::Error: From<ToqlError>;
        
    async fn load_many<T, B>(&mut self, query: B) -> Result<Vec<T>, Self::Error>
    where T: Load<Self::Row, Self::Error>, B: Borrow<Query<T>> + Send + Sync, <T as Keyed>::Key: FromRow<Self::Row, Self::Error>,
   <Self as ToqlApi>::Error: From<ToqlError> ;
    
    async fn load_page<T, B>(&mut self, query: B, page: Page) -> Result<(Vec<T>, Option<(u64, u64)>), Self::Error>
    where T: Load<Self::Row, Self::Error>, B: Borrow<Query<T>> + Send + Sync, <T as Keyed>::Key: FromRow<Self::Row, Self::Error>,
     <Self as ToqlApi>::Error: From<ToqlError>;

    async fn count<T, B>(&mut self, query: B) -> Result<u64, Self::Error>
    where T: Count, B: Borrow<Query<T>> + Send + Sync;

    async fn delete_one<K, B>(&mut self, key: B) -> Result<u64, Self::Error>
    where  B: Borrow<K> + Send, K: Key + ToQuery<<K as Key>::Entity> + Send, <K as Key>::Entity: Send,  <K as Key>::Entity: Delete;
    
    async fn delete_many<T, B>(&mut self, query: B) -> Result<u64, Self::Error>
    where T: Delete, B: Borrow<Query<T>> + Send + Sync;
    
    
}
