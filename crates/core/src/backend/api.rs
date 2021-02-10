use crate::{tree::{tree_predicate::TreePredicate, tree_map::TreeMap, tree_index::TreeIndex, tree_merge::TreeMerge, tree_insert::TreeInsert, tree_identity::TreeIdentity, tree_update::TreeUpdate}, keyed::Keyed, from_row::FromRow, sql_mapper::mapped::Mapped, error::ToqlError};

pub trait Load<R, E> : Keyed 
        + Mapped 
        + TreeMap 
        + FromRow<R,E>
        + TreePredicate
        + TreeIndex<R, E>
        + TreeMerge<R, E> + std::fmt::Debug 
        where <Self as Keyed>::Key: FromRow<R,E>, E: std::convert::From<ToqlError>{}

pub trait Insert :  TreeInsert + Mapped + TreeIdentity{}
pub trait Update :  TreeUpdate + Mapped + TreeIdentity + TreePredicate + TreeInsert{}

pub trait Count:Keyed + Mapped + std::fmt::Debug {}

pub trait Delete: Mapped + TreeMap + std::fmt::Debug {}
