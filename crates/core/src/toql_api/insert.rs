use crate::{
    table_mapper::mapped::Mapped,
    tree::{tree_identity::TreeIdentity, tree_insert::TreeInsert, tree_map::TreeMap},
};






pub trait Insert: TreeInsert + Mapped + TreeIdentity +TreeMap + Send{}

