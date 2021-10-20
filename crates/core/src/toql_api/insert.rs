//! Convenient super trait for function [insert](crate::toql_api::ToqlApi::insert_many).

use crate::{
    table_mapper::mapped::Mapped,
    tree::{
        tree_identity::TreeIdentity, tree_insert::TreeInsert, tree_map::TreeMap,
        tree_predicate::TreePredicate,
    },
};

/// Bind generic types to this trait when writing database independend functions.
///
/// See similar example on [ToqlApi](crate::toql_api::ToqlApi)
/// and on [insert_many](crate::toql_api::ToqlApi::insert_many).
pub trait Insert: TreeInsert + Mapped + TreeIdentity + TreeMap + TreePredicate + Send {}
