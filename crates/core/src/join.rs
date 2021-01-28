pub mod from_row;
pub mod keyed;
pub mod tree_identity;
pub mod tree_index;
pub mod tree_insert;
pub mod tree_merge;
pub mod tree_predicate;
pub mod tree_update;

use std::{borrow::Cow, boxed::Box};

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde_feature",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Join<E: crate::key::Keyed> 
{
    Key(E::Key),
    Entity(Box<E>),
}

impl<E> Default for Join<E>
where
    E: Default + crate::key::Keyed,
{
    fn default() -> Self {
        Join::Entity(Box::new(E::default()))
    }

}
// TODO decide on how to display keys to user
/* impl<E> std::fmt::Display for Join<E>
where
    E:  std::fmt::Display + crate::key::Keyed,
    <E as crate::key::Keyed>::Key:  std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match self {
            Join::Key(k) => k.fmt(f),
            Join::Entity(e) => e.fmt(f),
        }
    }
} */

impl<E> Clone for Join<E>
where
    E: Clone + crate::key::Keyed,
    <E as crate::key::Keyed>::Key: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Join::Key(k) => Join::Key(k.clone()),
            Join::Entity(e) => Join::Entity(e.clone()),
        }
    }
}


impl<T> Join<T> where T: crate::key::Keyed {

/* pub fn new(key: <T as crate::key::Keyed>::Key) -> Self{
       Join::Key(key)
    } */


    pub fn entity(&self) -> Option<&T>{
        match self {
            Join::Key(_) => {None}
            Join::Entity(e) => {Some(&e)}
        }
    }
    pub fn mut_entity(&mut self) -> Option<&mut T>{
        match self {
            Join::Key(_) => {None}
            Join::Entity(e) => {Some(e.as_mut())}
        }
    }
    pub fn entity_or_err<E>(&self, err: E) -> std::result::Result<&T, E>{
        match self {
            Join::Key(_) => {Err(err)}
            Join::Entity(e) => {Ok(&e)}
        }
    }
    pub fn mut_entity_or_err<E>(&mut self, err: E) -> std::result::Result<&mut T, E>{
        match self {
            Join::Key(_) => {Err(err)}
            Join::Entity(e) => {Ok(e.as_mut())}
        }
    }

    pub fn key(&self) -> crate::error::Result<<T as crate::key::Keyed>::Key>
    where <T as crate::key::Keyed>::Key: std::clone::Clone
    {
        match self {
            Join::Entity(e) => Ok(e.try_get_key()?),
            Join::Key(k) => { Ok(k.to_owned()) }
        }
    }

}

// Trait to improve convinience when working with Join<T> and Option<Join<T>>
pub trait TryJoin where Self: Sized{

    type Output;
    fn try_join(&self) -> crate::error::Result<&Self::Output>;
    fn try_join_or<E>(&self, err: E) -> std::result::Result<&Self::Output, E> {
        self.try_join().map_err(|_| err)
    }
}

impl<T> TryJoin for Join<T> where T: crate::key::Keyed {
    type Output =  T;
    fn try_join(&self) ->  crate::error::Result<&Self::Output> {
        self.entity_or_err(crate::error::ToqlError::NotFound)
    }
}
impl<T> TryJoin for Option<Join<T>> where T: crate::key::Keyed {
    type Output =  T;
    fn try_join(&self) ->  crate::error::Result<&Self::Output> {
        self.as_ref().ok_or(crate::error::ToqlError::JoinExpected)?.entity_or_err(crate::error::ToqlError::JoinExpected)
    }
}
impl<T> TryJoin for Option<Option<Join<T>>> where T: crate::key::Keyed {
    type Output =  T;
    fn try_join(&self) ->  crate::error::Result<&Self::Output> {
         self.as_ref().ok_or(crate::error::ToqlError::JoinExpected)?
        .as_ref().ok_or(crate::error::ToqlError::JoinExpected)?
        .entity_or_err(crate::error::ToqlError::JoinExpected)
    }
}
