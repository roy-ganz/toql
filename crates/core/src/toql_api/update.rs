//! Convenient super trait for function [update](crate::toql_api::ToqlApi::update_many).
use crate::{
    table_mapper::mapped::Mapped,
    tree::{
        tree_identity::TreeIdentity, tree_insert::TreeInsert, tree_map::TreeMap,
        tree_predicate::TreePredicate, tree_update::TreeUpdate,
    },
};
/// Bind generic types to this trait when writing database independend functions.
///
/// See similar example on [ToqlApi](crate::toql_api::ToqlApi)
/// and on [update_many](crate::toql_api::ToqlApi::update_many).
pub trait Update:
    TreeUpdate + Mapped + TreeIdentity + TreePredicate + TreeInsert + TreeMap + Send + Sync
{
}
