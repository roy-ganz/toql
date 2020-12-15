use crate::parameter_map::ParameterMap;
use crate::sql_expr::SqlExpr;

pub trait JoinHandler {
    /// Return customized SQL on predicate
    fn build_on_predicate(
        &self,
        on_predicate: SqlExpr,
        _aux_params: &ParameterMap,
    ) -> Result<SqlExpr, crate::sql_builder::sql_builder_error::SqlBuilderError> {
        Ok(on_predicate)
    }
}

impl std::fmt::Debug for (dyn JoinHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
