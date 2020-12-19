pub mod from_row;
pub mod keyed;
pub mod tree_identity;
pub mod tree_index;
pub mod tree_insert;
pub mod tree_merge;
pub mod tree_predicate;
pub mod tree_update;

use std::boxed::Box;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde_feature",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Join<E: crate::key::Keyed> {
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
