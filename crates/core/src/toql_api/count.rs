//! Convenient super trait for function [count](crate::toql_api::ToqlApi::count).
use crate::{keyed::Keyed, table_mapper::mapped::Mapped, tree::tree_map::TreeMap};

/// Bind generic types to this trait when writing database independend functions.
///
/// See similar example on [ToqlApi](crate::toql_api::ToqlApi)
/// and on [count](crate::toql_api::ToqlApi::count).
pub trait Count: Keyed + Mapped + TreeMap {}
