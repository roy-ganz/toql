//! Convenient super trait for function [load](crate::toql_api::ToqlApi::load_many).

use crate::{
    from_row::FromRow,
    keyed::Keyed,
    table_mapper::mapped::Mapped,
    tree::{
        tree_index::TreeIndex, tree_map::TreeMap, tree_merge::TreeMerge,
        tree_predicate::TreePredicate,
    },
};

/// Bind generic types to this trait when writing database independend functions.
///
/// See example on [ToqlApi](crate::toql_api::ToqlApi)
/// and on [load_many](crate::toql_api::ToqlApi::load_many).
/// Must be bound with the row and error type of the database backend.
pub trait Load<R, E>:
    Keyed + Mapped + TreeMap + FromRow<R, E> + TreePredicate + TreeIndex<R, E> + TreeMerge<R, E> + Send
{
}
