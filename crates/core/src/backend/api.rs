use crate::{
    error::ToqlError,
    from_row::FromRow,
    keyed::Keyed,
    sql_mapper::mapped::Mapped,
    tree::{
        tree_identity::TreeIdentity, tree_index::TreeIndex, tree_insert::TreeInsert,
        tree_map::TreeMap, tree_merge::TreeMerge, tree_predicate::TreePredicate,
        tree_update::TreeUpdate,
    },
};

pub trait Load<R, E>:
    Keyed
    + Mapped
    + TreeMap
    + FromRow<R, E>
    + TreePredicate
    + TreeIndex<R, E>
    + TreeMerge<R, E>
    + std::fmt::Debug
where
    <Self as Keyed>::Key: FromRow<R, E>,
    E: std::convert::From<ToqlError>,
{
}

pub trait Insert: TreeInsert + Mapped + TreeIdentity {}
pub trait Update: TreeUpdate + Mapped + TreeIdentity + TreePredicate + TreeInsert {}

pub trait Count: Keyed + Mapped + std::fmt::Debug {}

pub trait Delete: Mapped + TreeMap + std::fmt::Debug {}
