use crate::{
    table_mapper::mapped::Mapped,
    tree::{
        tree_identity::TreeIdentity, tree_insert::TreeInsert, tree_map::TreeMap,
        tree_predicate::TreePredicate, tree_update::TreeUpdate,
    },
};

pub trait Update:
    TreeUpdate + Mapped + TreeIdentity + TreePredicate + TreeInsert + TreeMap + Send + Sync
{
}
