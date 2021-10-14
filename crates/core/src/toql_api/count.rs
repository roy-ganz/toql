use crate::{keyed::Keyed, table_mapper::mapped::Mapped, tree::tree_map::TreeMap};

pub trait Count: Keyed + Mapped + TreeMap {}
