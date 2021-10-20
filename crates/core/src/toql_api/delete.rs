//! Convenient super trait for function [delete](crate::toql_api::ToqlApi::delete_many).
use crate::{table_mapper::mapped::Mapped, tree::tree_map::TreeMap};

/// Bind generic types to this trait when writing database independend functions.
///
/// See similar example on [ToqlApi](crate::toql_api::ToqlApi)
/// and on [delete_many](crate::toql_api::ToqlApi::delete_many).
pub trait Delete: Mapped + TreeMap {}
