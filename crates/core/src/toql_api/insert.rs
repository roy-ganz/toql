use crate::{
    table_mapper::mapped::Mapped,
    tree::{
        tree_identity::TreeIdentity, tree_insert::TreeInsert, tree_map::TreeMap,
        tree_predicate::TreePredicate,
    },
};

pub trait Insert: TreeInsert + Mapped + TreeIdentity + TreeMap + TreePredicate + Send {}
