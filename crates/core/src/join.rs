//! Join struct to simplify update handling of joins
/// A join struct can contain either the full entity or just it's key.
/// This allows to load the full entity. For updates however,
/// you are not forced to set a full entity if you only want to  
/// updating a foreign key.
///
/// ### Compare both
///
/// ``` ignore
/// use toql::prelude::Join;
///
/// #[derive(Toql)]
/// struct User {
///    #[toql(key)]
///     id: u64,
///     #[toql(join())] {
///     language: Join<Language>
///     #[toql(join())] {
///     country: Country
///  }
///
/// For loading both `language` and `country` behave the same.
/// The difference comes on updating: Lets assume a web interface
/// that can change both `language` and `country`.
/// For `language`, the web can only send back the key. It will deserialize into the join.
/// To change `country` however the client needs to send back a full valid country,
/// otherwise the deserializer (serde) will fail.
/// Likewise when programming `Join` is more ergonomic in update situations.
///
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

// The Join struct that hold either an entity or it's key.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde_feature",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde_feature", serde(untagged))]
pub enum Join<E: crate::keyed::Keyed> {
    /// Full entity is held. The entity is wrapped inside a `Box`. That does allow
    /// circular dependencies, in theory. In practice the compiler goes wild :(
    Entity(Box<E>),
    /// The entities key
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
    /// Constructs join for entity
    pub fn with_entity(entity: T) -> Self {
        Join::Entity(Box::new(entity))
    }

    /// Constructs join for key
    pub fn with_key(key: impl Into<<T as crate::keyed::Keyed>::Key>) -> Self {
        Join::Key(key.into())
    }

    /// Returns entity or `None`, if key is held.
    pub fn entity(&self) -> Option<&T> {
        match self {
            Join::Key(_) => None,
            Join::Entity(e) => Some(&e),
        }
    }

    /// Returns mutable entity or `None`, if key is held.
    pub fn entity_mut(&mut self) -> Option<&mut T> {
        match self {
            Join::Key(_) => None,
            Join::Entity(e) => Some(e.as_mut()),
        }
    }

    /// Returns entity or error `E`, if key is held.
    pub fn entity_or_err<E>(&self, err: E) -> std::result::Result<&T, E> {
        match self {
            Join::Key(_) => Err(err),
            Join::Entity(e) => Ok(&e),
        }
    }
    /// Returns mut entity or error `E`, if key is held.
    pub fn entity_mut_or_err<E>(&mut self, err: E) -> std::result::Result<&mut T, E> {
        match self {
            Join::Key(_) => Err(err),
            Join::Entity(e) => Ok(e.as_mut()),
        }
    }

    /// Returns a key. If entity is held, key is taken from that entity
    pub fn key(&self) -> <T as crate::keyed::Keyed>::Key
    where
        <T as crate::keyed::Keyed>::Key: std::clone::Clone,
    {
        match self {
            Join::Entity(e) => e.key(),
            Join::Key(k) => k.to_owned(),
        }
    }

    /// Unwraps join into its entity. Can fail
    pub fn into_entity(self) -> std::result::Result<T, ToqlError> {
        match self {
            Join::Key(_) => Err(ToqlError::NotFound),
            Join::Entity(e) => Ok(*e),
        }
    }
}
