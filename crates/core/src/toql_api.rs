//! The common interface for all database backends.
use async_trait::async_trait;
use std::{
    borrow::{Borrow, BorrowMut},
    result::Result,
};

use crate::{
    error::ToqlError, from_row::FromRow, key::Key, keyed::Keyed, page::Page,
    page_counts::PageCounts, query::Query,
};

use count::Count;
use delete::Delete;
use fields::Fields;
use insert::Insert;
use load::Load;
use paths::Paths;
use update::Update;

pub mod count;
pub mod delete;
pub mod fields;
pub mod insert;
pub mod load;
pub mod paths;
pub mod update;

/// Every database provider implements this trait.
/// This ensures that the user of Toql can easily switch databases.
///
/// The trait can also be used to write database independend code. This however requires more trait bounds
/// and is usually less ergonomic than passing for a specific database type to the functions.
///
/// ### Example
/// The following code shows the function signature to load an entity `MyUser` from a any database:
///
/// ```
/// use toql::prelude::{ToqlError, ToqlApi, Load, FromRow};
///
/// async fn load_user<R, E, A>(toql: &mut A) -> std::result::Result<Vec<MyUser>, MyError>
/// where A: ToqlApi<Row=R, Error = E>,
///     E: From<ToqlError>,
///     MyUser: Load<R, E>,
///     <MyUser as Keyed>::Key: FromRow<R, E>,  // Needed until rust-lang/rfcs#2289 is resolved
///     MyError: From<E>
/// {
///        let users = toql.load_many().await?;
///        Ok(users)
///  }
/// ```
#[async_trait]
pub trait ToqlApi
where
    Self::Error: From<ToqlError>,
{
    type Row;
    type Error;

    async fn insert_one<T>(&mut self, entity: &mut T, paths: Paths) -> Result<(), Self::Error>
    where
        T: Insert;

    async fn insert_many<T, Q>(
        &mut self,
        entities: &mut [Q],
        paths: Paths,
    ) -> Result<(), Self::Error>
    where
        T: Insert,
        Q: BorrowMut<T> + Send;

    async fn update_one<T>(&mut self, entity: &mut T, fields: Fields) -> Result<(), Self::Error>
    where
        T: Update + Keyed;

    async fn update_many<T, Q>(
        &mut self,
        entities: &mut [Q],
        fields: Fields,
    ) -> Result<(), Self::Error>
    where
        T: Update + Keyed,
        Q: BorrowMut<T> + Send + Sync;

    async fn load_one<T, B>(&mut self, query: B) -> Result<T, Self::Error>
    where
        T: Load<Self::Row, Self::Error>,
        B: Borrow<Query<T>> + Send + Sync,
        <T as Keyed>::Key: FromRow<Self::Row, Self::Error>;

    async fn load_many<T, B>(&mut self, query: B) -> Result<Vec<T>, Self::Error>
    where
        T: Load<Self::Row, Self::Error>,
        B: Borrow<Query<T>> + Send + Sync,
        <T as Keyed>::Key: FromRow<Self::Row, Self::Error>;

    async fn load_page<T, B>(
        &mut self,
        query: B,
        page: Page,
    ) -> Result<(Vec<T>, Option<PageCounts>), Self::Error>
    where
        T: Load<Self::Row, Self::Error>,
        B: Borrow<Query<T>> + Send + Sync,
        <T as Keyed>::Key: FromRow<Self::Row, Self::Error>;

    async fn count<T, B>(&mut self, query: B) -> Result<u64, Self::Error>
    where
        T: Count,
        B: Borrow<Query<T>> + Send + Sync;

    async fn delete_one<K>(&mut self, key: K) -> Result<(), Self::Error>
    where
        K: Key + Into<Query<<K as Key>::Entity>> + Send,
        <K as Key>::Entity: Send + Delete;

    async fn delete_many<T, B>(&mut self, query: B) -> Result<(), Self::Error>
    where
        T: Delete,
        B: Borrow<Query<T>> + Send + Sync;
}
