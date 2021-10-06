//! A JoinHandler may modify an ON condition in a join clause.
/// Use it to create or build an ON condition.
/// To disable a join, just return `false`.
///
/// TODO proper example
/// Notice that aux_params may not only come from context or a query,
/// but also from that mapped join. This is to give additional building context
/// 
/// ## Example (see full working example in tests)
/// ``` ignore
/// use toql::query::FieldFilter;
/// use toql::join_handler::JoinHandler;
/// use toql::sql_builder::SqlBuilderError;
/// struct MyHandler {};
/// 
/// impl JoinHandler for MyHandler {
///     fn build_on_predicate(&self, on_predicate: SqlExpr, aux_params: &ParameterMap,)
///     ->Result<Option<SqlExpr>, SqlBuilderError> {
///        --snip--
///     }
/// }
///
/// #[derive(Toql)]
/// #[toql(auto_key = true)]
/// struct User{
///    id:u64
///    #[toql(join( join_handler="handler()", aux_params(name="hint", value="42")))]
///    language : Language
/// }
///
use crate::parameter_map::ParameterMap;
use crate::sql_expr::{resolver::Resolver, SqlExpr};
use crate::sql_builder::sql_builder_error::SqlBuilderError;
use std::{fmt, marker::{Sync, Send}};

pub trait JoinHandler {
    /// Returns customized SQL on predicate
    fn build_on_predicate(
        &self,
        on_predicate: SqlExpr,
        aux_params: &ParameterMap,
    ) -> Result<SqlExpr, SqlBuilderError> {
         let expr = Resolver::resolve_aux_params(on_predicate, aux_params);
         Ok(expr)
    }
}

impl std::fmt::Debug for (dyn JoinHandler + Send + Sync + 'static) {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JoinHandler()")
    }
}

/// Handles the standart filters as documented in the guide.
/// Returns [FilterInvalid](../sql_builder/enum.SqlBuilderError.html) for any attempt to use FN filters.
#[derive(Debug, Clone)]
pub struct DefaultJoinHandler {}

impl DefaultJoinHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl JoinHandler for DefaultJoinHandler {}

impl Default for DefaultJoinHandler {
    fn default() -> Self {
        Self::new()
    }
}
