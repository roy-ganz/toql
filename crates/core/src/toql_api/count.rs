

use crate::{
    keyed::Keyed,
    sql_mapper::mapped::Mapped,
    tree::tree_map::TreeMap, 
};

pub trait Count: Keyed + Mapped + TreeMap + std::fmt::Debug {}

