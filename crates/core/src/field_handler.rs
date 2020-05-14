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
///     fn build_filter(&self, sql: &str, _filter: &FieldFilter)
///     ->Result<Option<String>, SqlBuilderError> {
///        --snip--
///     }
///     fn build_param(&self, _filter: &FieldFilter) -> Vec<String> {
///         --snip--
///     }
/// }
/// let my_handler = MyHandler {};
/// let mapper = SqlMapper::new_with_handler(my_handler);
///

use std::collections::HashMap;
 use crate::query::field_filter::FieldFilter;
 
 use crate::sql_builder::sql_builder_error::SqlBuilderError;

use crate::sql::{Sql, SqlArg};
use crate::parameter::Parameter;

pub trait FieldHandler {
    
    /// Context parameters allow to share information between different handlers
    

    /// Return sql and params if you want to select it.
    fn build_select(
        &self,
        select: Sql,
        _aux_params: &Parameter,
    ) -> Result<Option<Sql>, crate::sql_builder::sql_builder_error::SqlBuilderError> {
        Ok(Some(select))
    }

    /// Match filter and return SQL expression.
    /// Do not insert parameters in the SQL expression, use `?` instead.
    /// If you miss some arguments, raise an error, typically `SqlBuilderError::FilterInvalid`
    fn build_filter(
        &self,
        select: Sql,
        filter: &FieldFilter,
        aux_params: &Parameter,
    ) -> Result<Option<Sql>, crate::sql_builder::sql_builder_error::SqlBuilderError>;
   
    
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
        mut select: Sql,
        filter: &FieldFilter,
        _aux_params: &HashMap<String, SqlArg>,
    ) -> Result<Option<Sql>, crate::sql_builder::sql_builder_error::SqlBuilderError> {
        match filter {
            FieldFilter::Eq(criteria) => Ok(Some((format!("{} = ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::Eqn => Ok(Some((format!("{} IS NULL", select.0), select.1))),
            FieldFilter::Ne(criteria) => Ok(Some((format!("{} <> ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::Nen => Ok(Some((format!("{} IS NOT NULL", select.0), select.1))),
            FieldFilter::Ge(criteria) => Ok(Some((format!("{} >= ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::Gt(criteria) => Ok(Some((format!("{} > ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::Le(criteria) => Ok(Some((format!("{} <= ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::Lt(criteria) => Ok(Some((format!("{} < ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::Bw(lower, upper) => Ok(Some((format!("{} BETWEEN ? AND ?", select.0), {
                select.1.push(lower.clone());
                select.1.push(upper.clone());
                select.1
            }))),
            FieldFilter::Re(criteria) => Ok(Some((format!("{} RLIKE ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
            FieldFilter::In(args) => Ok(Some((
                format!(
                    "{} IN ({})",
                    select.0,
                    std::iter::repeat("?")
                        .take(args.len())
                        .collect::<Vec<&str>>()
                        .join(",")
                ),
                {
                   // let a: Vec<SqlArg> = args.iter( ).collect();
                    select.1.extend_from_slice(&args);
                    select.1
                },
            ))),
            FieldFilter::Out(args) => Ok(Some((
                format!(
                    "{} NOT IN ({})",
                    select.0,
                    std::iter::repeat("?")
                        .take(args.len())
                        .collect::<Vec<&str>>()
                        .join(",")
                ),
                {
                    //let a: Vec<SqlArg> = args.iter().collect::<Vec<_>>();
                    select.1.extend_from_slice(&args);
                    select.1
                },
            ))),
            //      FieldFilter::Sc(_) => Ok(Some(format!("FIND_IN_SET (?, {})", expression))),
            FieldFilter::Lk(criteria) => Ok(Some((format!("{} LIKE ?", select.0), {
                select.1.push(criteria.clone());
                select.1
            }))),
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