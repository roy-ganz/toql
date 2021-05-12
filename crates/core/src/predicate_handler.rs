use crate::{
    parameter_map::ParameterMap, sql_arg::SqlArg, sql_builder::sql_builder_error::SqlBuilderError,
    sql_expr::SqlExpr,
};

pub trait PredicateHandler {
    /// Match filter and return SQL expression or None, if no filtering is required.
    /// Do not insert parameters in the SQL expression, use `?` instead and provide the argument in the vector.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_predicate(
        &self,
        expression: SqlExpr,
        args: &Vec<SqlArg>,
        aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, SqlBuilderError>;
}

impl std::fmt::Debug for (dyn PredicateHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PredicateHandler()")
    }
}

pub struct DefaultPredicateHandler;

impl DefaultPredicateHandler {
    pub fn new() -> Self {
        DefaultPredicateHandler {}
    }
}

impl PredicateHandler for DefaultPredicateHandler {
    fn build_predicate(
        &self,
        predicate: SqlExpr,
        _args: &Vec<SqlArg>,
        _aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, crate::sql_builder::sql_builder_error::SqlBuilderError> {
        // Wrap in parens
        let mut e = SqlExpr::literal("(");
        e.extend(predicate);
        e.push_literal(")");
        Ok(Some(e))
    }
}
