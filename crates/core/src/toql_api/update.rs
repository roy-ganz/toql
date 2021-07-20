use crate::{
    tree::{tree_identity::TreeIdentity, tree_update::TreeUpdate, 
    tree_predicate::TreePredicate, tree_insert::TreeInsert, tree_map::TreeMap}, 
    sql_mapper::mapped::Mapped
};

pub trait Update: TreeUpdate + Mapped + TreeIdentity + TreePredicate + TreeInsert + TreeMap + Send + Sync{}
