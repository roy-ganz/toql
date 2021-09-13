pub mod from_row;
pub mod keyed;

pub mod tree_identity;
pub mod tree_index;
pub mod tree_insert;
pub mod tree_merge;
pub mod tree_predicate;
pub mod tree_update;

use crate::error::ToqlError;

use std::boxed::Box;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde_feature",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde_feature", serde(untagged))]
pub enum Join<E: crate::keyed::Keyed> {
    Entity(Box<E>),
    Key(E::Key),
}

impl<E> Default for Join<E>
where
    E: Default + crate::keyed::Keyed,
{
    fn default() -> Self {
        Join::Entity(Box::new(E::default()))
    }
}
// TODO decide on how to display keys to user
/* impl<E> std::fmt::Display for Join<E>
where
    E:  std::fmt::Display + crate::keyed::Keyed,
    <E as crate::keyed::Keyed>::Key:  std::fmt::Display,
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
    E: Clone + crate::keyed::Keyed,
    <E as crate::keyed::Keyed>::Key: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Join::Key(k) => Join::Key(k.clone()),
            Join::Entity(e) => Join::Entity(e.clone()),
        }
    }
}

impl<T> Join<T>
where
    T: crate::keyed::Keyed,
{
    pub fn with_entity(entity: T) -> Self{
       Join::Entity(Box::new(entity))
    } 
    pub fn with_key(key: impl Into<<T as crate::keyed::Keyed>::Key>) -> Self{
       Join::Key(key.into())
    } 

    pub fn entity(&self) -> Option<&T> {
        match self {
            Join::Key(_) => None,
            Join::Entity(e) => Some(&e),
        }
    }
    pub fn entity_mut(&mut self) -> Option<&mut T> {
        match self {
            Join::Key(_) => None,
            Join::Entity(e) => Some(e.as_mut()),
        }
    }
    pub fn entity_or_err<E>(&self, err: E) -> std::result::Result<&T, E> {
        match self {
            Join::Key(_) => Err(err),
            Join::Entity(e) => Ok(&e),
        }
    }
    pub fn entity_mut_or_err<E>(&mut self, err: E) -> std::result::Result<&mut T, E> {
        match self {
            Join::Key(_) => Err(err),
            Join::Entity(e) => Ok(e.as_mut()),
        }
    }

    pub fn key(&self) -> <T as crate::keyed::Keyed>::Key
    where
        <T as crate::keyed::Keyed>::Key: std::clone::Clone,
    {
        match self {
            Join::Entity(e) => e.key(),
            Join::Key(k) => k.to_owned(),
        }
    }

    pub fn into_entity(self) -> std::result::Result<T, ToqlError> {
        match self {
            Join::Key(_) => Err(ToqlError::NotFound),
            Join::Entity(e) => Ok(*e),
        }
    }
}
