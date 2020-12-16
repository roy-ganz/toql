pub mod keyed;
pub mod tree_identity;
pub mod tree_merge;
pub mod tree_predicate;
pub mod tree_update;
pub mod tree_insert;
pub mod tree_index;
pub mod from_row;

use std::boxed::Box;


#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde_feature", derive(serde::Serialize, serde::Deserialize))]
pub enum Join<E: crate::key::Keyed> {
    Key(E::Key),
    Entity(Box<E>),
}




