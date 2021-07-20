
use crate::{
    error::ToqlError,
    from_row::FromRow,
    keyed::Keyed,
    sql_mapper::mapped::Mapped,
    tree::{
        tree_index::TreeIndex, 
        tree_map::TreeMap, tree_merge::TreeMerge, tree_predicate::TreePredicate,
    } 
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
    + Send
where
    <Self as Keyed>::Key: FromRow<R, E>,
    E: std::convert::From<ToqlError>,
{
}

