
use crate::{
    from_row::FromRow,
    keyed::Keyed,
    table_mapper::mapped::Mapped,
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
    + Send
    
{
   
}

