use crate::query::field_path::Descendents;
use std::result::Result;
use crate::{sql_expr::SqlExpr, sql_builder::sql_builder_error::SqlBuilderError};

pub trait TreeKeys
{
    fn keys<'a>(
        descendents: &mut Descendents<'a>,
        field: &str,
        key_expr: &mut SqlExpr,
    ) -> Result<(), SqlBuilderError>;
}
