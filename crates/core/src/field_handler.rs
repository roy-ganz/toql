use crate::parameter_map::ParameterMap;
/// A FieldHandler maps a Toql field onto an SQL.
/// Use it to
/// - define your own custom function (through FN)
/// - map the standart filters differently
/// - disallow standart filters
/// - handle fields that do not exist in the struct
/// - handle fields that match multiple columns (full text index)
///
/// ## Example (see full working example in tests)
/// ``` ignore
/// use toql::query::FieldFilter;
/// use toql::sql_mapper::FieldHandler;
/// use toql::sql_builder::SqlBuilderError;
/// struct MyHandler {};
///
/// impl FieldHandler for MyHandler {
///     fn build_filter(&self, sql: &SqlExpr, _filter: &FieldFilter)
///     ->Result<Option<SqlExpr>, SqlBuilderError> {
///        --snip--
///     }
/// }
/// let my_handler = MyHandler {};
/// let mapper = SqlMapper::new_with_handler(my_handler);
///
use crate::query::field_filter::FieldFilter;
use crate::sql_builder::sql_builder_error::SqlBuilderError;
use crate::sql_expr::SqlExpr;

pub trait FieldHandler {
    /// Context parameters allow to share information between different handlers

    /// Return sql and params if you want to select it.
    fn build_select(
        &self,
        select: SqlExpr,
        _aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        Ok(Some(select))
    }

    /// Match filter and return SQL expression.
    /// Do not insert parameters in the SQL expression, use `?` instead.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_filter(
        &self,
        select: SqlExpr,
        filter: &FieldFilter,
        aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, SqlBuilderError>;
}

impl std::fmt::Debug for (dyn FieldHandler + std::marker::Send + std::marker::Sync + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FieldHandler()")
    }
}
/*
pub fn sql_param(s: String) -> String {
    if s.chars().next().unwrap_or(' ') == '\'' {
        return unquote(&s).expect("Argument invalid"); // Must be valid, because Pest rule
    }
    s
} */

impl FieldHandler for BasicFieldHandler {
    fn build_filter(
        &self,
        mut select: SqlExpr,
        filter: &FieldFilter,
        _aux_params: &ParameterMap,
    ) -> Result<Option<SqlExpr>, SqlBuilderError> {
        match filter {
            FieldFilter::Eq(criteria) => {
                select.push_literal(" = ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Eqn => {
                select.push_literal(" IS NULL");
                Ok(Some(select))
            }
            FieldFilter::Ne(criteria) => {
                select.push_literal(" <> ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Nen => {
                select.push_literal(" IS NULL");
                Ok(Some(select))
            }
            FieldFilter::Ge(criteria) => {
                select.push_literal(" >= ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Gt(criteria) => {
                select.push_literal(" > ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Le(criteria) => {
                select.push_literal(" <= ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Lt(criteria) => {
                select.push_literal(" < ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Bw(lower, upper) => {
                select
                    .push_literal(" BETWEEN ")
                    .push_arg(lower.clone())
                    .push_literal(" AND ")
                    .push_arg(upper.clone());
                Ok(Some(select))
            }
            FieldFilter::Re(criteria) => {
                select.push_literal(" RLIKE ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::In(args) => {
                select.push_literal(" IN (");
                for a in args {
                    select.push_separator(", ");
                    select.push_arg(a.clone());
                }
                select.push_literal(")");
                Ok(Some(select))
            }
            FieldFilter::Out(args) => {
                select.push_literal(" NOT IN (");
                for a in args {
                    select.push_separator(", ");
                    select.push_arg(a.clone());
                }
                select.push_literal(")");
                Ok(Some(select))
            }
            //      FieldFilter::Sc(_) => Ok(Some(format!("FIND_IN_SET (?, {})", expression))),
            FieldFilter::Lk(criteria) => {
                select.push_literal(" LIKE ").push_arg(criteria.clone());
                Ok(Some(select))
            }
            FieldFilter::Fn(name, _) => Err(SqlBuilderError::FilterInvalid(name.to_owned())), // Must be implemented by user
        }
    }
}

/// Handles the standart filters as documented in the guide.
/// Returns [FilterInvalid](../sql_builder/enum.SqlBuilderError.html) for any attempt to use FN filters.
#[derive(Debug, Clone)]
pub struct BasicFieldHandler {}

impl BasicFieldHandler {
    pub fn new() -> Self {
        Self {}
    }
}
