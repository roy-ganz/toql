
use crate::{tree::tree_map::TreeMap, sql_mapper::mapped::Mapped};
      
pub trait Delete: Mapped + TreeMap + std::fmt::Debug {}
